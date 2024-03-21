use crate::{BLOCKS, BLOCK_CONFIGS, PARAMS, PORTS, STRINGS};
use micrortu_build_utils::{Block, Direction, IEType, Port};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, Token,
};

struct RegisterBlockInput {
    block_type: Ident,
    block_name: Ident,
    factory_fn: Ident,
    init_fn: Ident,
    step_fn: Ident,
}

impl Parse for RegisterBlockInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let block_type = input.parse()?;
        input.parse::<Token![,]>()?;
        let block_name = input.parse()?;
        input.parse::<Token![,]>()?;
        let factory_fn = input.parse()?;
        input.parse::<Token![,]>()?;
        let init_fn = input.parse()?;
        input.parse::<Token![,]>()?;
        let step_fn = input.parse()?;

        Ok(Self {
            block_type,
            block_name,
            factory_fn,
            init_fn,
            step_fn,
        })
    }
}

#[allow(clippy::too_many_lines)]
pub fn register_block(input: TokenStream) -> TokenStream {
    let RegisterBlockInput {
        block_type,
        block_name,
        factory_fn,
        init_fn,
        step_fn,
    } = parse_macro_input!(input as RegisterBlockInput);

    let block_name_str = block_name.to_string();
    if block_name_str.len() < 2 {
        return syn::Error::new_spanned(
            block_name,
            "Block name must be at least 2 characters long",
        )
        .to_compile_error()
        .into();
    }
    if block_name_str.len() > 32 {
        return syn::Error::new_spanned(
            block_name,
            "Block name must be at most 32 characters long",
        )
        .to_compile_error()
        .into();
    }
    if block_name_str.starts_with('_') {
        return syn::Error::new_spanned(block_name, "Block name cannot start with underscore")
            .to_compile_error()
            .into();
    }
    if block_name_str.chars().any(|c| !c.is_lowercase()) {
        return syn::Error::new_spanned(block_name, "Block name must be all lowercase")
            .to_compile_error()
            .into();
    }

    let module_name = Ident::new(&format!("_block_{block_name}"), block_name.span());
    let factory_name = Ident::new(&format!("factory_{block_name}"), block_name.span());
    let init_name = Ident::new(&format!("init_{block_name}"), block_name.span());
    let step_name = Ident::new(&format!("step_{block_name}"), block_name.span());
    let ports_static_name = Ident::new(&format!("PORTS_{block_name}"), block_name.span());
    let params_static_name = Ident::new(&format!("PARAMS_{block_name}"), block_name.span());

    let ports = PORTS.lock().expect("poison");
    let params = PARAMS.lock().expect("poison");
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let key = (block_name_str.clone(), crate_name.clone());
    let Some(ports) = ports.get(&key).cloned() else {
        return syn::Error::new_spanned(block_name, "Missing ports for block")
            .into_compile_error()
            .into();
    };
    let Some(params) = params.get(&key).cloned() else {
        return syn::Error::new_spanned(block_name, "Missing params for block")
            .into_compile_error()
            .into();
    };
    let block = Block {
        name: block_name_str.to_string(),
        description: String::new(),
        semver_requirement: None,
        ports: ports.clone(),
        params: params.clone(),
        block_conf: BLOCK_CONFIGS
            .lock()
            .expect("poison")
            .get(&block_name_str)
            .cloned(),
    };
    BLOCKS.lock().expect("poison").insert(key, block);
    let ports = to_quote(ports);
    let params = to_quote(params);

    let output = quote! {
        mod #module_name {
            use ::micrortu_sdk::{Shared, StepResult, BindingDefinition};

            use super::#factory_fn as factory_fn;
            use super::#init_fn as init_fn;
            use super::#step_fn as step_fn;
            use super::#block_type as _BLOCK_TYPE;

            #[no_mangle]
            extern "C" fn #factory_name(shared: &mut Shared) -> Option<&'static mut _BLOCK_TYPE> {
                factory_fn(shared)
            }
            #[no_mangle]
            extern "C" fn #init_name(shared: &mut Shared, block: &mut _BLOCK_TYPE) -> StepResult {
                init_fn(shared, block)
            }
            #[no_mangle]
            extern "C" fn #step_name(shared: &mut Shared, block: &mut _BLOCK_TYPE) -> StepResult {
                step_fn(shared, block)
            }
            #[allow(non_upper_case_globals)]
            #[no_mangle]
            static #ports_static_name: &[BindingDefinition] = #ports;
            #[allow(non_upper_case_globals)]
            #[no_mangle]
            static #params_static_name: &[BindingDefinition] = #params;
        }
    };
    output.into()
}

fn intern_static_string(s: &str) -> u16 {
    let mut strings = STRINGS.lock().expect("poison");
    let len = strings.len();
    strings.push_str(s);
    len.try_into().expect("too many strings interned")
}

fn to_quote(ports: Vec<Port>) -> impl ToTokens {
    let ports = ports.into_iter().map(|port| {
        let name = &port.name;
        let name_len = port.name.len() as u8;
        let name_offset = intern_static_string(&port.name);
        let flags = port.required as u8;
        let min_size = port.min.get();
        let to_nonzero_max_size = port.max.map_or(0, |m| m.get());
        let typ = match port.typ {
            IEType::TI1 => 1u8,
            IEType::TI3 => 3,
            IEType::TI13 => 13,
            IEType::TI45 => 45,
            IEType::TI50 => 50,
            IEType::TI112 => 112,
        };
        let direction_quote = match port.direction {
            Direction::In => quote! { ::micrortu_sdk::IN },
            Direction::Out => quote! { ::micrortu_sdk::OUT },
            Direction::InOut => quote! { ::micrortu_sdk::IN_OUT },
        };

        quote! { {
        #[cfg(target_arch = "wasm32")] {
            ::micrortu_sdk::BindingDefinition {
                name_offset: #name_offset,
                name_len: #name_len,
                typ: #typ,
                flags: #flags,
                min_size: #min_size,
                max_size: ::core::num::NonZeroU8::new(#to_nonzero_max_size),
                direction: #direction_quote,
            }
        }
        #[cfg(not(target_arch = "wasm32"))] {
            const NAME: &str = stringify!(#name);
            ::micrortu_sdk::NativeBindingDefinition::<'static> {
                name: &NAME,
                flags: #flags,
                typ: #typ,
                min_size: #min_size,
                max_size: ::core::num::NonZeroU8::new(#to_nonzero_max_size),
                direction: #direction_quote,
            }
        }

        }}
    });

    quote! { &[#(#ports),*] }
}
