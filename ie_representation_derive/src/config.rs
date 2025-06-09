use micrortu_build_utils::{AllowedType, BlockConf};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Meta, MetaList, Type};

use crate::{
    bindings::parse_block_names,
    state::{set_block_conf, should_bail_on_duplicates},
};

pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut block_names = vec![];
    for attr in &input.attrs {
        match &attr.meta {
            Meta::List(MetaList { path, tokens, .. })
                if path.get_ident().map_or(false, |i| *i == "block_names") =>
            {
                parse_block_names(tokens.clone().into(), &mut block_names);
            }
            _ => (),
        }
    }
    if block_names.is_empty() {
        return syn::Error::new_spanned(input, "Config must have #[block_names(...)] attribute")
            .to_compile_error()
            .into();
    }
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields
            .named
            .into_iter()
            .map(|f| {
                let field_name = f.ident.unwrap().to_string();
                let field_type = map_type_to_allowed(&f.ty);
                (field_name, field_type)
            })
            .collect(),
        _ => {
            return syn::Error::new_spanned(
                name,
                "Config can only be derived for structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    let block_conf = BlockConf { fields };

    for block_name in block_names {
        let res = set_block_conf(&block_name, block_conf.clone());

        if res.is_some() && should_bail_on_duplicates() {
            return syn::Error::new_spanned(name, "Config with that name already exists")
                .to_compile_error()
                .into();
        }
    }

    let init_fn_name = format!("_init_{name}");
    let init_fn_name = syn::Ident::new(&init_fn_name, name.span());
    let is_native = cfg!(feature = "micrortu_sdk_internal");
    let baked_in = block_conf
        .fields
        .iter()
        .map(|(field_name, field_type)| {
            let field_type_ident = match field_type {
                AllowedType::U8 => quote! { ::micrortu_build_utils::AllowedType::U8 },
                AllowedType::U16 => quote! { ::micrortu_build_utils::AllowedType::U16 },
                AllowedType::U32 => quote! { ::micrortu_build_utils::AllowedType::U32 },
                AllowedType::U64 => quote! { ::micrortu_build_utils::AllowedType::U64 },
                AllowedType::I8 => quote! { ::micrortu_build_utils::AllowedType::I8 },
                AllowedType::I16 => quote! { ::micrortu_build_utils::AllowedType::I16 },
                AllowedType::I32 => quote! { ::micrortu_build_utils::AllowedType::I32 },
                AllowedType::I64 => quote! { ::micrortu_build_utils::AllowedType::I64 },
                AllowedType::F32 => quote! { ::micrortu_build_utils::AllowedType::F32 },
                AllowedType::F64 => quote! { ::micrortu_build_utils::AllowedType::F64 },
            };
            quote! { (#field_name.into(), #field_type_ident) }
        });

    let baked_in = if is_native {
        quote! {
            #[cfg(feature = "std")]
            fn config_schema() -> ::micrortu_build_utils::BlockConf {
                ::micrortu_build_utils::BlockConf {
                    fields: vec![#(#baked_in),*],
                }
            }
        }
    } else {
        quote! {}
    };

    let output = quote! {
        impl ::micrortu_sdk::Config for #name { #baked_in }

        #[allow(dead_code)]
        #[allow(non_snake_case)]
        fn #init_fn_name() {
            if false {
                let config: #name = unsafe { ::core::mem::zeroed() };
            }
        }
    };

    output.into()
}

fn map_type_to_allowed(ty: &Type) -> AllowedType {
    match ty {
        Type::Path(type_path) if type_path.path.segments.len() == 1 => {
            let segment = &type_path.path.segments[0].ident;
            match segment.to_string().as_str() {
                "u8" => AllowedType::U8,
                "u16" => AllowedType::U16,
                "u32" => AllowedType::U32,
                "u64" => AllowedType::U64,
                "i8" => AllowedType::I8,
                "i16" => AllowedType::I16,
                "i32" => AllowedType::I32,
                "i64" => AllowedType::I64,
                "f32" => AllowedType::F32,
                "f64" => AllowedType::F64,
                _ => panic!("Unsupported type for Config: {segment}"),
            }
        }
        _ => panic!("Unsupported type for Config"),
    }
}
