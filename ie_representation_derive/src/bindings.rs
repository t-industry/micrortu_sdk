use std::num::NonZeroU8;

use crate::{PARAMS, PORTS};
use micrortu_build_utils::{Direction, IEType};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Attribute, Ident, LitInt, Meta, MetaList, Token, Visibility,
};

struct Port {
    attrs: Vec<Attribute>,
    mode: Ident,
    name: Ident,
    typ: IEType,
    lower_bound: usize,
    upper_bound: Option<usize>,
    optional: bool,
}

impl Parse for Port {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let attrs = input.call(Attribute::parse_outer)?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let typ: Ident = input.parse()?;
        let typ = match typ.to_string().as_str() {
            "TI1" => IEType::TI1,
            "TI3" => IEType::TI3,
            "TI13" => IEType::TI13,
            "TI45" => IEType::TI45,
            "TI50" => IEType::TI50,
            "TI112" => IEType::TI112,
            _ => {
                return Err(syn::Error::new(
                    typ.span(),
                    "Unknown type. Supported types are TI1, TI3, TI13, TI45, TI50, TI112",
                ))
            }
        };
        let mode: Ident = input.parse()?;

        let left: LitInt = input.parse()?;
        let lower_bound = left.base10_parse::<usize>()?;
        if lower_bound == 0 {
            return Err(syn::Error::new(
                left.span(),
                "Lower bound must be greater than zero",
            ));
        }

        let right: Result<LitInt, _> = input.parse();
        let upper_bound = match right {
            Ok(right) => {
                let upper_bound = right.base10_parse::<usize>()?;
                if upper_bound == 0 {
                    return Err(syn::Error::new(
                        right.span(),
                        "Upper bound must be greater than zero",
                    ));
                }
                Some(upper_bound)
            }
            Err(_) => None,
        };

        if lower_bound > upper_bound.unwrap_or(lower_bound) {
            return Err(syn::Error::new(
                left.span(),
                "Upper bound must be greater than or equal to lower bound",
            ));
        }

        let optional = input.parse::<Token![?]>().is_ok();

        Ok(Self {
            attrs,
            mode,
            name,
            typ,
            lower_bound,
            upper_bound,
            optional,
        })
    }
}

struct PortsInput {
    attrs: Vec<Attribute>,
    block_names: Vec<String>,
    visibility: Visibility,
    struct_name: Ident,
    ports: Punctuated<Port, Token![,]>,
}

struct BlockNames {
    names: Vec<String>,
}

impl Parse for BlockNames {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let names: Punctuated<Ident, _> = input.parse_terminated(|it| it.parse(), Token![,])?;
        let names = names.iter().map(|i| i.to_string()).collect();
        Ok(Self { names })
    }
}

pub fn parse_block_names(tokens: TokenStream, names: &mut Vec<String>) -> TokenStream {
    let parsed = parse_macro_input!(tokens as BlockNames);
    *names = parsed.names;
    quote! {}.into()
}

impl Parse for PortsInput {
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let mut attrs = input.call(Attribute::parse_outer)?;
        let mut block_names = vec![];
        let position_of_block_name = attrs.iter().position(|a| match &a.meta {
            Meta::List(MetaList { path, tokens, .. })
                if path.get_ident().map_or(false, |i| *i == "block_names") =>
            {
                parse_block_names(tokens.clone().into(), &mut block_names);
                !block_names.is_empty()
            }
            _ => false,
        });
        match position_of_block_name {
            Some(position) => attrs.remove(position),
            None => {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "block_names attribute is required",
                ));
            }
        };
        let visibility: Visibility = input.parse()?;
        input.parse::<Token![struct]>()?;
        let struct_name: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let ports = content.parse_terminated(|it| it.parse(), Token![,])?;

        Ok(Self {
            attrs,
            block_names,
            visibility,
            struct_name,
            ports,
        })
    }
}

#[allow(clippy::too_many_lines)]
pub fn bindings(input: TokenStream, is_ports: bool) -> TokenStream {
    let PortsInput {
        attrs,
        block_names,
        visibility,
        struct_name,
        ports,
    } = parse_macro_input!(input as PortsInput);

    let mut report_blocks = vec![];
    let mut parse_blocks = vec![];
    let mut names = vec![];
    let mut meta_bindings = vec![];

    for port in ports {
        let name = &port.name;
        let mode_str = port.mode.to_string();
        let is_single = port.lower_bound == 1 && port.upper_bound == Some(1);
        let is_optional = port.optional;
        let attrs = &port.attrs;
        let typ = match (mode_str.as_str(), is_optional, is_single) {
            ("In", true, true) => quote! { ::micrortu_sdk::GetSingleOptional },
            ("In", false, true) => quote! { ::micrortu_sdk::GetSingle },
            ("In", true, false) => quote! { ::micrortu_sdk::GetMultipleOptional },
            ("In", false, false) => quote! { ::micrortu_sdk::GetMultiple },

            ("Out", true, true) => quote! { ::micrortu_sdk::SetSingleOptional },
            ("Out", false, true) => quote! { ::micrortu_sdk::SetSingle },
            ("Out", true, false) => quote! { ::micrortu_sdk::SetMultipleOptional },
            ("Out", false, false) => quote! { ::micrortu_sdk::SetMultiple },

            ("InOut", true, true) => quote! { ::micrortu_sdk::GetSetSingleOptional },
            ("InOut", false, true) => quote! { ::micrortu_sdk::GetSetSingle },
            ("InOut", true, false) => quote! { ::micrortu_sdk::GetSetMultipleOptional },
            ("InOut", false, false) => quote! { ::micrortu_sdk::GetSetMultiple },
            _ => {
                return syn::Error::new_spanned(port.mode, format!("Unknown port mode: {mode_str}"))
                    .to_compile_error()
                    .into()
            }
        };

        names.push(quote! {
            #(#attrs)*
            pub #name: #typ<'a>
        });

        let min_size = port.lower_bound as u8;
        let max_size = port
            .upper_bound
            .map_or(quote! { None }, |m| quote! { Some(#m as u8) });
        let flags = !is_optional as u16;

        let direction = match mode_str.as_str() {
            "In" => Direction::In,
            "Out" => Direction::Out,
            "InOut" => Direction::InOut,
            _ => {
                return syn::Error::new_spanned(port.mode, format!("Unknown port mode: {mode_str}"))
                    .to_compile_error()
                    .into()
            }
        };

        let name_str = name.to_string();

        let binding = micrortu_build_utils::Port {
            name: name_str.clone(),
            typ: port.typ,
            description: String::new(),
            direction,
            required: !is_optional,
            min: NonZeroU8::new(min_size).unwrap(),
            max: port.upper_bound.and_then(|m| NonZeroU8::new(m as u8)),
        };

        meta_bindings.push(binding);

        let to_nonzero_max_size = port.upper_bound.map_or(0, |m| m as u8);
        let direction_quote = match direction {
            Direction::In => quote! { ::micrortu_sdk::IN },
            Direction::Out => quote! { ::micrortu_sdk::OUT },
            Direction::InOut => quote! { ::micrortu_sdk::IN_OUT },
        };

        report_blocks.push(quote! { {
            #[cfg(not(target_arch = "wasm32"))] {
                const NAME: &str = stringify!(#name);
                ::micrortu_sdk::NativeBindingDefinition::<'static> {
                    name: &NAME,
                    flags: #flags,
                    min_size: #min_size,
                    max_size: ::core::num::NonZeroU8::new(#to_nonzero_max_size),
                    direction: #direction_quote,
                }
            } }
        });

        parse_blocks.push(quote! {#name: {
            let (len, dirty) = ::micrortu_sdk::parse_port(&source, &cursor, &dirty, #is_optional, #min_size, #max_size)?;
            let (this, new_source) = source.split_at_mut(len);
            source = new_source
                .get_mut(1..)
                .ok_or(::micrortu_sdk::ParseError::NotTerminated)?; // skip null terminator
            cursor = cursor.wrapping_add(len).wrapping_add(1); // skip null terminator
            #typ::new(this, dirty)
          },
        });
    }

    for block_name in block_names {
        let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
        let store = if is_ports { &PORTS } else { &PARAMS };
        let mut store = store.lock().expect("poison");
        let prev = store.insert((block_name.to_string(), crate_name), meta_bindings.clone());

        if prev.is_some() {
            return syn::Error::new_spanned(
                block_name.clone(),
                format!("Bindings are already defined for block `{block_name}`"),
            )
            .to_compile_error()
            .into();
        }
    }

    let parse = quote! {
        fn parse(mut source: &'a mut [::micrortu_sdk::IEBuf], dirty: &'a mut [u8; 8])
            -> Result<Self, ::micrortu_sdk::ParseError>
        {
            let dirty = ::core::cell::Cell::from_mut(dirty);
            let mut cursor = 0;
            Ok(Self {
                _marker: ::core::marker::PhantomData,
                #(#parse_blocks)*
            })
        }
    };

    let report = quote! {
        #[cfg(not(target_arch = "wasm32"))]
        pub const fn report() -> &'static [::micrortu_sdk::NativeBindingDefinition<'static>] {
            const BINDINGS: &[::micrortu_sdk::NativeBindingDefinition<'static>] = &[
                #(#report_blocks,)*
            ];
            BINDINGS
        }
    };

    let impl_comment = format!(
        " Auto-generated by `{krate}::ports`.\n",
        krate = env!("CARGO_PKG_NAME")
    );
    let impl_doc_comment = quote!(#[doc=#impl_comment]);

    let expanded = quote! {
        #impl_doc_comment
        #(#attrs)*
        #[derive(Debug)]
        #visibility struct #struct_name<'a> {
            #(#names,)*
            _marker: ::core::marker::PhantomData<&'a ()>,
        }

        impl<'a> #struct_name<'a> {
            #parse
            #report
        }
    };

    expanded.into()
}
