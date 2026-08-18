use micrortu_build_utils::Block;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse::Parser, punctuated::Punctuated, LitStr, Path, Token};

use crate::state::{set_block, set_block_conf, set_params, set_ports, BlockFns, Loc};

/// Expands `import_blocks!(lib_a::pid, lib_b::twv_ctrl)` into invocations of
/// the replay macros that `export_blocks!` or `finalize!` exported from those
/// crates.
pub fn import_blocks(input: TokenStream) -> TokenStream {
    match try_import_blocks(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

fn try_import_blocks(input: TokenStream) -> syn::Result<TokenStream> {
    let paths: Vec<Path> = Punctuated::<Path, Token![,]>::parse_terminated
        .parse(input)?
        .into_iter()
        .collect();
    if paths.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "import_blocks! takes paths to blocks, e.g. import_blocks!(lib_a::pid)",
        ));
    }
    let mut invocations = Vec::new();
    for path in paths {
        let mut segments: Vec<_> = path.segments.iter().map(|s| s.ident.clone()).collect();
        let block = segments.pop().expect("path has at least one segment");
        if segments.is_empty() {
            return Err(syn::Error::new_spanned(
                &path,
                "import_blocks! paths must name the crate and the block, \
                 e.g. import_blocks!(lib_a::pid)",
            ));
        }
        let leading = path.leading_colon;
        let replay = format_ident!("__micrortu_import_{block}", span = block.span());
        invocations.push(quote! { #leading #(#segments)::* :: #replay!(); });
    }
    Ok(quote! { #(#invocations)* }.into())
}

/// Hidden entry point invoked by the replay macros: registers one exported
/// block, given as the json encoding of its metadata, into the current
/// crate's registry.
pub fn replay_meta(input: TokenStream) -> TokenStream {
    match try_replay_meta(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

fn try_replay_meta(input: TokenStream) -> syn::Result<TokenStream> {
    let lit: LitStr = syn::parse(input)?;
    let block: Block = serde_json::from_str(&lit.value())
        .map_err(|e| syn::Error::new(lit.span(), format!("malformed block metadata: {e}")))?;

    let name = block.name.clone();
    let loc = Loc::of(lit.span());
    let conflict = |what: &str, prev: Loc| {
        syn::Error::new(
            lit.span(),
            format!("imported block `{name}` conflicts with the {what} defined at {prev}"),
        )
    };
    set_ports(&name, block.ports, loc.clone()).map_err(|prev| conflict("ports", prev))?;
    set_params(&name, block.params, loc.clone()).map_err(|prev| conflict("params", prev))?;
    if let Some(conf) = block.block_conf {
        set_block_conf(&name, conf, loc.clone()).map_err(|prev| conflict("config", prev))?;
    }
    set_block(&name, BlockFns::imported(), loc).map_err(|prev| conflict("block", prev))?;
    Ok(TokenStream::new())
}
