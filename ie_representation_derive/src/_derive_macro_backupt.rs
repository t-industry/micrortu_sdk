use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DeriveInput, Expr, ExprLit, Field, Fields, GenericArgument, Lit,
    LitInt, Meta, MetaNameValue, Type, Visibility,
};

extern crate proc_macro;

#[repr(u8)]
enum Direction {
    In,
    Out,
    InOut,
}

impl quote::ToTokens for Direction {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::In => tokens.extend(quote::quote! { ::types::Direction::In }),
            Self::Out => tokens.extend(quote::quote! { ::types::Direction::Out }),
            Self::InOut => tokens.extend(quote::quote! { ::types::Direction::InOut }),
        }
    }
}

#[proc_macro_derive(IEvalueBindings, attributes(min_size, max_size))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_parser(input) {
        Ok(token_stream) => token_stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn derive_parser(input: DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    let struct_name = &input.ident;

    // Ensure the derive is used on a struct with named fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "IEvalueBindings can only be used with structs with named fields.",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "IEvalueBindings can only be used with structs.",
            ))
        }
    };

    let (parsing_blocks, report_blocks) = process_fields(fields)?;

    if !fields.iter().all(|f| f.ident.is_some()) {
        return Err(syn::Error::new_spanned(
            input,
            "IEvalueBindings can only be used with structs with named fields.",
        ));
    }

    let parsed_fields = fields.iter().enumerate().map(|(i, f)| {
        let block = &parsing_blocks[i];
        let name = f.ident.as_ref().unwrap();
        quote! { #name: #block? }
    });

    let (impl_generics, struct_generics, where_clause) = input.generics.split_for_impl();
    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics ::types::IEvalueBindings<'a> for #struct_name #struct_generics
            #where_clause
        {
            fn parse(mut ir: &'a mut [::types::IEValue]) -> Result<Self, ::types::ParseError> {
                Ok(Self {
                    #(#parsed_fields,)*
                })
            }
            fn report() -> &'static [::types::BindingDefinition] {
                &[
                    #(#report_blocks,)*
                ]
            }
        }
    };

    Ok(expanded)
}

struct Attributes {
    min_size: Option<u16>,
    max_size: Option<u16>,
}

impl TryFrom<&[syn::Attribute]> for Attributes {
    type Error = syn::Error;
    fn try_from(attrs: &[syn::Attribute]) -> Result<Self, Self::Error> {
        let mut min_size = None;
        let mut max_size = None;

        for attr in attrs {
            let value = match &attr.meta {
                Meta::NameValue(MetaNameValue {
                    value:
                        Expr::Lit(ExprLit {
                            lit: Lit::Int(lit), ..
                        }),
                    ..
                }) => lit.base10_parse::<u16>().map_err(|_| ()),
                _ => Err(()),
            };
            let Ok(value) = value else {
                return Err(syn::Error::new_spanned(
                    attr,
                    "Expected a literal integer for min_size",
                ));
            };
            if attr.path().is_ident("min_size") {
                min_size = Some(value);
            } else if attr.path().is_ident("max_size") {
                max_size = Some(value);
            }
        }

        Ok(Self { min_size, max_size })
    }
}

fn get_cardinality(ty: &syn::Type) -> Result<bool, syn::Error> {
    let target_type = match ty {
        syn::Type::Reference(ty) => &*ty.elem,
        ty => ty,
    };
    let cardinality = matches!(ty, syn::Type::Array(_) | syn::Type::Slice(_));
    let target_type = match target_type {
        syn::Type::Path(_) => Some(target_type),
        syn::Type::Array(array) => Some(&*array.elem),
        syn::Type::Slice(slice) => Some(&*slice.elem),
        _ => None,
    };

    let result = match target_type {
        Some(syn::Type::Path(path)) => path.path.is_ident("IEValue"),
        _ => false,
    };

    if result {
        Ok(cardinality)
    } else {
        Err(syn::Error::new_spanned(
            ty,
            "Expected the type to be IEValue",
        ))
    }
}

fn process_fields(
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
) -> Result<(Vec<TokenStream2>, Vec<TokenStream2>), syn::Error> {
    let mut parse_blocks = vec![];
    let mut bindings = vec![];

    for field in fields {
        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "Expected the field to have a name"))?;

        let Attributes { min_size, max_size } = Attributes::try_from(field.attrs.as_slice())?;
        let (direction, optional, cardinality) = parse_optional_direction(&field.ty)?;
        let min_size = min_size.unwrap_or(1).max(1);
        let max_size = match (max_size, cardinality) {
            (Some(max_size), _) => Some(max_size),
            (None, true) => None,
            (None, false) => Some(1),
        };

        let mmax_size = max_size.map_or(quote! { None }, |m: u16| quote! { Some(#m) });
        let mut flags = 0u16;
        if optional {
            flags |= 1;
        }

        bindings.push(quote! {
            ::types::BindingDefinition {
                name: stringify!(#field_name),
                flags: #flags,
                min_size: #min_size,
                max_size: #mmax_size,
                direction: #direction,
            }
        });

        let collect = quote! {
            let mut len = 0;
            if !ir.is_empty() {
                while ir.get(len).ok_or(::types::ParseError::NotTerminated)?[0] != 0 {
                    len += 1;
                }
            }
        };
        let optional_early_return = if optional {
            quote! {
                if len == 0 {
                    break 'block Ok(None);
                }
            }
        } else {
            quote! {}
        };
        let bounds_check = quote! {
            if len < #min_size as usize {
                return Err(::types::ParseError::NotEnoughData);
            }
            if #mmax_size.map_or(false, |m: u16| len > m as usize) {
                return Err(::types::ParseError::TooMuchData);
            }
        };
        let split_ir = quote! {
            let (values, new_ir) = ir.split_at_mut(len);
            ir = &mut new_ir[1..];
        };
        let singular_check = match max_size {
            Some(1) => quote! {
                if len > 1 {
                    return Err(::types::ParseError::MultiplePointsForSingular);
                }
            },
            _ => quote! {},
        };

        let mutable = !matches!(direction, Direction::In);

        let ret = if mutable {
            match (max_size, optional) {
                (Some(1), true) => quote! { Ok(Some(&mut values[0])) },
                (Some(1), false) => quote! { Ok(&mut values[0]) },
                (_, true) => quote! { Ok(Some(values)) },
                (_, false) => quote! { Ok(values) },
            }
        } else {
            match (max_size, optional) {
                (Some(1), true) => quote! { Ok(Some(&values[0])) },
                (Some(1), false) => quote! { Ok(&values[0]) },
                (_, true) => quote! { Ok(Some(values)) },
                (_, false) => quote! { Ok(values) },
            }
        };

        parse_blocks.push(quote! {
            // #[allow(unused_labels)]
            'block: {
                #collect
                #optional_early_return
                #bounds_check
                #split_ir
                #singular_check
                #ret
            }
        });
    }

    Ok((parse_blocks, bindings))
}

fn parse_optional_direction(ty: &Type) -> Result<(Direction, bool, bool), syn::Error> {
    let get_direction = |ty: &syn::Type| match ty {
        syn::Type::Reference(r) => {
            if r.mutability.is_none() {
                return Ok(Direction::In);
            }

            match &*r.elem {
                syn::Type::Path(path)
                    if path
                        .path
                        .segments
                        .last()
                        .map_or(false, |s| s.ident == "Option") =>
                {
                    Ok(Direction::Out)
                }
                _ => Ok(Direction::InOut),
            }
        }
        _ => Err(syn::Error::new_spanned(ty, "Expected a reference type")),
    };

    match ty {
        syn::Type::Reference(_) => Ok((get_direction(ty)?, false, get_cardinality(ty)?)),
        syn::Type::Path(_) => {
            let inner = extract_type_from_option(ty).ok_or(syn::Error::new_spanned(
                ty,
                "Expected the type to be a reference or an Option",
            ))?;

            Ok((get_direction(inner)?, true, get_cardinality(inner)?))
        }

        _ => Err(syn::Error::new_spanned(
            ty,
            "Expected the type to be a reference or an Option",
        )),
    }
}

fn extract_type_from_option(ty: &Type) -> Option<&Type> {
    fn path_is_option(path: &syn::Path) -> bool {
        path.segments.last().map_or(false, |s| s.ident == "Option")
    }

    match ty {
        Type::Path(typepath) if typepath.qself.is_none() && path_is_option(&typepath.path) => {
            // Get the first segment of the path (there is only one, in fact: "Option"):
            let type_params = &typepath.path.segments.first()?.arguments;
            // It should have only on angle-bracketed param ("<String>"):
            let generic_arg = match type_params {
                syn::PathArguments::AngleBracketed(params) => params.args.first()?,
                _ => return None,
            };
            // This argument must be a type:
            match generic_arg {
                GenericArgument::Type(ty) => Some(ty),
                _ => None,
            }
        }
        _ => None,
    }
}

/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////

use syn::{parse::Parse, parse::ParseStream, Ident};

struct Port {
    mode: Ident,
    name: Ident,
    lower_bound: usize,
    upper_bound: Option<usize>,
}

impl Parse for Port {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let name: Ident = input.parse()?;
        let mode: Ident = input.parse()?;

        let left: LitInt = input.parse()?;
        let lower_bound = left.base10_parse::<usize>()?;

        let right: Result<LitInt, _> = input.parse();
        let upper_bound = match right {
            Ok(right) => Some(right.base10_parse::<usize>()?),
            Err(_) => None,
        };

        if lower_bound > upper_bound.unwrap_or(lower_bound) {
            return Err(syn::Error::new(
                left.span(),
                "Upper bound must be greater than or equal to lower bound",
            ));
        }

        Ok(Self {
            mode,
            name,
            lower_bound,
            upper_bound,
        })
    }
}

struct PortsInput {
    visibility: Visibility,
    struct_name: Ident,
    ports: Vec<Port>,
}

impl Parse for PortsInput {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let visibility: Visibility = input.parse()?;
        let struct_name: Ident = input.parse()?;
        let mut ports = Vec::new();
        while !input.is_empty() {
            ports.push(input.parse()?);
        }

        Ok(Self {
            visibility,
            struct_name,
            ports,
        })
    }
}

#[proc_macro]
pub fn ports(input: TokenStream) -> TokenStream {
    let PortsInput {
        visibility,
        struct_name,
        ports,
    } = parse_macro_input!(input as PortsInput);

    let mut getters = vec![];
    let mut setters = vec![];
    let mut report_blocks = vec![];
    let mut parse_blocks = vec![];
    let ports_len = ports.len();

    for (index, port) in ports.into_iter().enumerate() {
        let name = &port.name;
        let getter_name = format_ident!("get_{}", name);
        let setter_name = format_ident!("set_{}", name);
        let mode = &port.mode;
        let mode_str = port.mode.to_string();

        let is_single = port.lower_bound <= 1 && port.upper_bound == Some(1);
        let is_optional = port.lower_bound == 0;

        let min_size = port.lower_bound as u16;
        let max_size = port
            .upper_bound
            .map_or(quote! { None }, |m| quote! { Some(#m as u16) });
        let flags = is_optional as u16;

        report_blocks.push(quote! {
            ::types::BindingDefinition {
                name: stringify!(#name),
                flags: #flags,
                min_size: #min_size,
                max_size: #max_size,
                direction: ::types::Direction::#mode,
            }
        });

        parse_blocks.push(quote! {{
            let mut len = 0;
            if ir.get(cursor + len).is_some() {
                while ir.get(cursor + len).ok_or(::types::ParseError::NotTerminated)?[0] != 0 {
                    len += 1;
                }
            }
            if len < #min_size as usize {
                let n = stringify!(#name);
                eprintln!("Not enough data for {n}");
                eprintln!("len: {len}");
                let m = #min_size;
                eprintln!("min_size: {m}");
                return Err(::types::ParseError::NotEnoughData);
            }
            if #max_size.map_or(false, |m: u16| len > m as usize) {
                return Err(::types::ParseError::TooMuchData);
            }
            indexies[#index] = (cursor, cursor + len);
            cursor += len + 1; // skip null terminator
        }});

        let getter = match (is_single, is_optional) {
            (true, true) => quote! {
                #visibility fn #getter_name(&self) -> Option<&IEValue> {
                    if self.indexies[#index].0 == self.indexies[#index].1 {
                        return None;
                    }
                    Some(&self.source[self.indexies[#index].0])
                }
            },

            (true, false) => quote! {
                #visibility fn #getter_name(&self) -> &IEValue {
                    &self.source[self.indexies[#index].0]
                }
            },

            (false, true) => quote! {
                #visibility fn #getter_name(&self) -> Option<&[IEValue]> {
                    if self.indexies[#index].0 == self.indexies[#index].1 {
                        return None;
                    }
                    Some(&self.source[self.indexies[#index].0..self.indexies[#index].1])
                }
            },

            (false, false) => quote! {
                #visibility fn #getter_name(&self) -> &[IEValue] {
                    &self.source[self.indexies[#index].0..self.indexies[#index].1]
                }
            },
        };

        #[cfg(disabled)]
        let setter = match (is_single, is_optional) {
            (true, true) => quote! {
                #visibility fn #setter_name<T>(&mut self, value: T)
                    where T: ::num_traits::ToPrimitive + ::num_traits::Num
                {
                    if self.indexies[#index].0 != self.indexies[#index].1 {
                        self.source[self.indexies[#index].0].try_update_from(value);
                    }
                }
            },
            (true, false) => quote! {
                #visibility fn #setter_name<T>(&mut self, value: T)
                    where T: ::num_traits::ToPrimitive + ::num_traits::Num
                {
                    self.source[self.indexies[#index].0].try_update_from(value);
                }
            },
            (false, false | true) => quote! {
                #visibility fn #setter_name<T>(&mut self, mut value: impl Iterator<Item = T>)
                    where T: ::num_traits::ToPrimitive + ::num_traits::Num
                {
                    for i in self.indexies[#index].0..self.indexies[#index].1 {
                        let Some(next) = value.next() else { return };
                        if self.source[i].try_update_from(next).is_err() {
                            return;
                        }
                    }
                }
            },
        };

        let setter = match (is_single, is_optional) {
            (true, true) => quote! {
                #visibility fn #setter_name(&mut self, value: IEValue) {
                    if self.indexies[#index].0 != self.indexies[#index].1 {
                        self.source[self.indexies[#index].0] = value;
                    }
                }
            },
            (true, false) => quote! {
                #visibility fn #setter_name(&mut self, value: IEValue) {
                    self.source[self.indexies[#index].0] = value;
                }
            },
            (false, false | true) => quote! {
                #visibility fn #setter_name(&mut self, mut value: impl Iterator<Item = IEValue>) {
                    for i in self.indexies[#index].0..self.indexies[#index].1 {
                        let Some(next) = value.next() else { return };
                        self.source[i] = next;
                    }
                }
            },
        };

        match mode_str.as_str() {
            "In" => {
                getters.push(getter);
            }
            "Out" => {
                setters.push(setter);
            }
            "InOut" => {
                getters.push(getter);
                setters.push(setter);
            }
            _ => {
                return syn::Error::new_spanned(port.mode, format!("Unknown port mode: {mode_str}"))
                    .to_compile_error()
                    .into()
            }
        }
    }

    let parse = quote! {
        fn parse(mut ir: &'a mut [::types::IEValue]) -> Result<Self, ::types::ParseError> {
            let mut indexies = [(0, 0); #ports_len];
            let mut cursor = 0;
            #(#parse_blocks)*
            Ok(Self {
                source: ir,
                indexies,
            })
        }
    };

    let report = quote! {
        fn report() -> &'static [::types::BindingDefinition] {
            &[
                #(#report_blocks,)*
            ]
        }
    };

    let expanded = quote! {
        #[derive(Debug)]
        #visibility struct #struct_name<'a> {
            source: &'a mut [::types::IEValue],
            indexies: [(usize, usize); #ports_len],
        }

        impl<'a> Ports<'a> {
            #parse
            #report
            #(#getters)*
            #(#setters)*
        }
    };

    expanded.into()
}
