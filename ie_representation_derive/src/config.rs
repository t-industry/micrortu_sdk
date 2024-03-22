use micrortu_build_utils::{AllowedType, BlockConf};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Meta, MetaList, Type};

use crate::{bindings::parse_block_names, BLOCK_CONFIGS};

pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut block_names = vec![];
    for attr in input.attrs {
        match &attr.meta {
            Meta::List(MetaList { path, tokens, .. })
                if path.get_ident().map_or(false, |i| *i == "block_names") =>
            {
                parse_block_names(tokens.clone().into(), &mut block_names);
            }
            _ => (),
        }
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
    let mut confs = BLOCK_CONFIGS.lock().expect("poison");

    for block_name in block_names {
        let krate = std::env::var("CARGO_PKG_NAME").unwrap();
        let key = (block_name.to_string(), krate.clone());

        if confs.keys().any(|k| k == &key) {
            return syn::Error::new_spanned(name, "Config with that name already exists")
                .to_compile_error()
                .into();
        }

        if confs.insert(key, block_conf.clone()).is_some() {
            return syn::Error::new_spanned(name, "Configuration with that name already exists")
                .to_compile_error()
                .into();
        }
    }

    let init_fn_name = format!("_init_{name}");
    let init_fn_name = syn::Ident::new(&init_fn_name, name.span());
    let output = quote! {
        impl ::micrortu_sdk::Config for #name {}

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
