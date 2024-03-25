use std::num::NonZeroU8;

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

use crate::state::{set_params, set_ports};

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
        let is_multiple = port.lower_bound > 1 || port.upper_bound.is_none();
        let is_optional = port.optional;
        let attrs = &port.attrs;
        let typ = match port.typ {
            IEType::TI1 => quote! { M_SP_NA_1 },
            IEType::TI3 => quote! { M_DP_NA_1 },
            IEType::TI13 => quote! { M_ME_NE_1 },
            IEType::TI45 => quote! { C_SC_NA_1 },
            IEType::TI50 => quote! { C_SE_NC_1 },
            IEType::TI112 => quote! { P_ME_NC_1 },
        };
        let typ = quote! { ::micrortu_sdk::ie_base::#typ };

        names.push(match (is_multiple, is_optional) {
            (true, true) => quote! { #(#attrs)* pub #name: Option<&'a mut [#typ]> },
            (true, false) => quote! { #(#attrs)* pub #name: &'a mut [#typ] },
            (false, true) => quote! { #(#attrs)* pub #name: Option<&'a mut #typ> },
            (false, false) => quote! { #(#attrs)* pub #name: &'a mut #typ },
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

        let ret = match (is_multiple, is_optional) {
            (true, true) => quote! { Some(&mut value[..]) },
            (true, false) => quote! { &mut value[..] },
            (false, true) => quote! { Some(&mut value[0]) },
            (false, false) => quote! { &mut value[0] },
        };

        parse_blocks.push(quote! {#name: {
            let (pad_len, rest) = header.split_at_mut(2);
            header = rest;
            let pad = pad_len[0] as usize;
            let len = pad_len[1] as usize;
            let (data, rest) = source.split_at_mut(pad + len);
            source = rest;
            if len < #min_size as usize {
                return Err(::micrortu_sdk::ParseError::NotEnoughData);
            }
            if #max_size.map_or(false, |m: u8| len > m as usize) {
                return Err(::micrortu_sdk::ParseError::TooMuchData);
            }
            let value = <#typ as ::zerocopy::FromBytes>::mut_slice_from(&mut data[pad..]);
            let mut value = value.ok_or(::micrortu_sdk::ParseError::InvalidData)?;
            #ret
          },
        });
    }

    for block_name in block_names {
        let res = if is_ports {
            set_ports(&block_name, meta_bindings.clone())
        } else {
            set_params(&block_name, meta_bindings.clone())
        };

        if res.is_err() {
            return syn::Error::new_spanned(
                block_name.clone(),
                format!("Bindings are already defined for block `{block_name}`"),
            )
            .to_compile_error()
            .into();
        }
    }

    let header_size = parse_blocks.len() * 2;
    let parse = quote! {
        fn parse(mut source: &'a mut [u8]) -> Self {
            fn parse_inner<'b>(source: &'b mut [u8]) -> Result<#struct_name<'b>, ::micrortu_sdk::ParseError> {
                if source.len() < #header_size {
                    return Err(::micrortu_sdk::ParseError::BadHeader);
                }
                let (mut header, mut source) = source.split_at_mut(#header_size);
                Ok(#struct_name {
                    _marker: ::core::marker::PhantomData,
                    #(#parse_blocks)*
                })
            };

            match parse_inner(source) {
                Ok(binds) => binds,
                Err(err) => {
                    ::micrortu_sdk::error!("Failed to parse bindings: {:?}", err);
                    ::core::arch::wasm32::unreachable()
                }
            }
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
