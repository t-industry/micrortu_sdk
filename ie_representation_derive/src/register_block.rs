use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, Token,
};

use crate::state::{set_block, BlockFns, Loc};

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
    if block_name_str
        .chars()
        .any(|c| c.is_ascii_alphabetic() && !c.is_lowercase())
    {
        return syn::Error::new_spanned(block_name, "Block name must be all lowercase")
            .to_compile_error()
            .into();
    }

    let fns = BlockFns {
        block_type: block_type.to_string(),
        factory: factory_fn.to_string(),
        init: init_fn.to_string(),
        step: step_fn.to_string(),
    };
    if let Err(prev) = set_block(&block_name_str, fns, Loc::of(block_name.span())) {
        return syn::Error::new_spanned(
            block_name,
            format!("block `{block_name_str}` is already registered at {prev}"),
        )
        .to_compile_error()
        .into();
    }

    let module_name = Ident::new(&format!("_block_{block_name}"), block_name.span());
    let factory_name = Ident::new(&format!("factory_{block_name}"), block_name.span());
    let init_name = Ident::new(&format!("init_{block_name}"), block_name.span());
    let step_name = Ident::new(&format!("step_{block_name}"), block_name.span());

    let output = quote! {
        #[cfg(target_arch = "wasm32")]
        mod #module_name {
            use ::micrortu_sdk::{Shared, StepResult, FactoryInput};

            use super::#factory_fn as factory_fn;
            use super::#init_fn as init_fn;
            use super::#step_fn as step_fn;
            use super::#block_type as _BLOCK_TYPE;

            #[no_mangle]
            extern "C" fn #factory_name(shared: &FactoryInput) -> Option<&'static mut _BLOCK_TYPE> {
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
        }
    };
    output.into()
}
