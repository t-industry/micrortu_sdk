use std::collections::BTreeMap;

use micrortu_build_utils::{Block, Direction, Port, WasmMetadata};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use semver::Version;
use syn::{parse::Parser, punctuated::Punctuated, Ident, Token};

use crate::state::{get_block_conf, get_ports_params, mark_finalized, registered_blocks, Loc};

const MINIMUM_FIRMWARE_VERSION: (u8, u8, u8) = (0, 0, 0);

/// Interner for the `COLLECTED_STRINGS` table. Identical strings share one
/// slot.
#[derive(Default)]
struct StringTable {
    data: String,
    offsets: BTreeMap<String, u16>,
}

impl StringTable {
    fn intern(&mut self, s: &str) -> syn::Result<u16> {
        if let Some(&offset) = self.offsets.get(s) {
            return Ok(offset);
        }
        let too_long = || {
            syn::Error::new(
                Span::call_site(),
                "port and param names exceed the 64KiB COLLECTED_STRINGS limit",
            )
        };
        let offset: u16 = self.data.len().try_into().map_err(|_| too_long())?;
        if s.len() > usize::from(u16::MAX - offset) {
            return Err(too_long());
        }
        self.data.push_str(s);
        self.offsets.insert(s.to_string(), offset);
        Ok(offset)
    }
}

fn binding_tokens(
    ports: &[Port],
    strings: &mut StringTable,
) -> syn::Result<proc_macro2::TokenStream> {
    let bindings = ports
        .iter()
        .map(|port| {
            let name_offset = strings.intern(&port.name)?;
            let name_len = u8::try_from(port.name.len()).map_err(|_| {
                syn::Error::new(
                    Span::call_site(),
                    format!("port name `{}` is longer than 255 bytes", port.name),
                )
            })?;
            let flags = port.required as u8;
            let min_size = port.min.get();
            let max_size = port.max.map_or(0, |m| m.get());
            let typ = port.typ as u8;
            let direction = match port.direction {
                Direction::In => quote! { ::micrortu_sdk::IN },
                Direction::Out => quote! { ::micrortu_sdk::OUT },
                Direction::InOut => quote! { ::micrortu_sdk::IN_OUT },
            };
            Ok(quote! {
                ::micrortu_sdk::BindingDefinition {
                    name_offset: #name_offset,
                    name_len: #name_len,
                    typ: #typ,
                    flags: #flags,
                    min_size: #min_size,
                    max_size: ::core::num::NonZeroU8::new(#max_size),
                    direction: #direction,
                }
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    Ok(quote! { &[#(#bindings),*] })
}

pub fn finalize(input: TokenStream) -> TokenStream {
    expand(input, true)
}

pub fn export_blocks(input: TokenStream) -> TokenStream {
    expand(input, false)
}

fn expand(input: TokenStream, emit_symbols: bool) -> TokenStream {
    match try_expand(input, emit_symbols) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

fn try_expand(input: TokenStream, emit_symbols: bool) -> syn::Result<TokenStream> {
    let macro_name = if emit_symbols {
        "finalize!"
    } else {
        "export_blocks!"
    };
    let blocks = collect_blocks(input, macro_name)?;
    let replay_macros = replay_macro_tokens(&blocks)?;
    let symbols = if emit_symbols {
        symbol_tokens(blocks)?
    } else {
        quote! {}
    };
    Ok(quote! {
        #symbols
        #(#replay_macros)*
    }
    .into())
}

/// Joins the registered blocks with their ports, params and config, erroring
/// on any incomplete block.
fn collect_blocks(input: TokenStream, macro_name: &str) -> syn::Result<Vec<Block>> {
    if let Err(prev) = mark_finalized(Loc::of(Span::call_site())) {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("finalize! or export_blocks! is already invoked at {prev}"),
        ));
    }

    let expected: Vec<Ident> = Punctuated::<Ident, Token![,]>::parse_terminated
        .parse(input)?
        .into_iter()
        .collect();

    let registered = registered_blocks();
    check_block_list(&expected, &registered, macro_name)?;

    if registered.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            format!(
                "{macro_name} found no registered blocks; it must be invoked in the same \
                 crate as the register_block! or import_blocks! invocations, after all of \
                 them"
            ),
        ));
    }

    let mut blocks = Vec::new();
    for (name, loc) in &registered {
        let (ports, params) = get_ports_params(name);
        let missing = |kind: &str| {
            syn::Error::new(
                Span::call_site(),
                format!(
                    "block `{name}` (registered at {loc}) has no {kind} with \
                     #[block_names({name})] expanded before {macro_name}; define them \
                     and keep {macro_name} after all block macros"
                ),
            )
        };
        blocks.push(Block {
            name: name.clone(),
            description: String::new(),
            semver_requirement: None,
            ports: ports.ok_or_else(|| missing("ports!"))?,
            params: params.ok_or_else(|| missing("params!"))?,
            block_conf: get_block_conf(name),
        });
    }
    Ok(blocks)
}

/// One exported replay macro per block, so other crates can register this
/// crate's blocks into their own registry with `import_blocks!`.
fn replay_macro_tokens(blocks: &[Block]) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    blocks
        .iter()
        .map(|block| {
            let json = serde_json::to_string(block).expect("serialization error");
            let macro_ident = format_ident!("__micrortu_import_{}", block.name);
            Ok(quote! {
                #[doc(hidden)]
                #[macro_export]
                macro_rules! #macro_ident {
                    () => {
                        ::micrortu_sdk::__replay_meta! { #json }
                    };
                }
            })
        })
        .collect()
}

fn symbol_tokens(blocks: Vec<Block>) -> syn::Result<proc_macro2::TokenStream> {
    let mut strings = StringTable::default();
    let mut binding_statics = Vec::new();

    for block in &blocks {
        let name = &block.name;
        let ports_tokens = binding_tokens(&block.ports, &mut strings)?;
        let params_tokens = binding_tokens(&block.params, &mut strings)?;
        let ports_static = format_ident!("PORTS_{name}");
        let params_static = format_ident!("PARAMS_{name}");
        binding_statics.push(quote! {
            #[allow(non_upper_case_globals)]
            #[no_mangle]
            static #ports_static: &[::micrortu_sdk::BindingDefinition] = #ports_tokens;
            #[allow(non_upper_case_globals)]
            #[no_mangle]
            static #params_static: &[::micrortu_sdk::BindingDefinition] = #params_tokens;
        });
    }

    let final_string = strings.data;
    let len = final_string.len();
    let bytes_array = final_string.as_bytes().iter().map(|&b| quote! { #b });
    let bytes_array = quote! { [ #(#bytes_array),* ] };
    let doc = format!(" Collected strings: {final_string:?}");

    let sdk_version = Version::parse(env!("CARGO_PKG_VERSION")).expect("invalid version");
    let sdk_version = (
        sdk_version.major as u8,
        sdk_version.minor as u8,
        sdk_version.patch as u8,
    );

    let metadata = WasmMetadata {
        minimum_firmware_version: MINIMUM_FIRMWARE_VERSION,
        sdk_version,
        blocks,
    };

    let metadata = serde_json::to_string(&metadata).expect("serialization error");
    let metadata_len = metadata.len();
    let metadata_bytes_array = metadata.as_bytes().iter().map(|&b| quote! { #b });
    let metadata_bytes_array = quote! { [ #(#metadata_bytes_array),* ] };

    Ok(quote! {
        #[no_mangle]
        #[doc = #doc]
        static COLLECTED_STRINGS: [u8; #len] = #bytes_array;

        #[link_section = "metadata"]
        #[allow(dead_code)]
        static META: [u8; #metadata_len] = #metadata_bytes_array;

        #[cfg(target_arch = "wasm32")]
        const _: () = {
            #(#binding_statics)*
        };
    })
}

/// When the macro lists block names, requires the list to match the
/// registered blocks exactly, so a block that failed to register (wrong crate
/// or an invocation after the collecting macro) is reported instead of
/// silently missing from the metadata.
fn check_block_list(
    expected: &[Ident],
    registered: &[(String, Loc)],
    macro_name: &str,
) -> syn::Result<()> {
    if expected.is_empty() {
        return Ok(());
    }
    let mut error: Option<syn::Error> = None;
    let mut push = |err: syn::Error| match &mut error {
        Some(prev) => prev.combine(err),
        None => error = Some(err),
    };
    for ident in expected {
        let name = ident.to_string();
        if !registered.iter().any(|(n, _)| *n == name) {
            push(syn::Error::new(
                ident.span(),
                format!(
                    "block `{name}` is not registered in this crate; register_block! or \
                     import_blocks! must be invoked in the same crate, before {macro_name}"
                ),
            ));
        }
    }
    for (name, loc) in registered {
        if !expected.iter().any(|i| *i == name) {
            push(syn::Error::new(
                Span::call_site(),
                format!(
                    "block `{name}` (registered at {loc}) is missing from the \
                     {macro_name} block list"
                ),
            ));
        }
    }
    match error {
        Some(err) => Err(err),
        None => Ok(()),
    }
}
