use std::sync::Mutex;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Attribute, Ident, LitInt, Token, Visibility,
};

struct Port {
    attrs: Vec<Attribute>,
    mode: Ident,
    name: Ident,
    lower_bound: usize,
    upper_bound: Option<usize>,
    optional: bool,
}

impl Parse for Port {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let attrs = input.call(Attribute::parse_outer)?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
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
            lower_bound,
            upper_bound,
            optional,
        })
    }
}

struct PortsInput {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    struct_name: Ident,
    ports: Punctuated<Port, Token![,]>,
}

impl Parse for PortsInput {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility: Visibility = input.parse()?;
        input.parse::<Token![struct]>()?;
        let struct_name: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let ports = content.parse_terminated(|it| it.parse(), Token![,])?;

        Ok(Self {
            attrs,
            visibility,
            struct_name,
            ports,
        })
    }
}

/**
# Macros for generating parser of arguments block requires.

## Example

```rust
ports! {
    pub struct Ports {
        // parameter `a` has minimum size of 1 and unbouded maximum size, required
        a: In 1,
        // parameter `y` should have exactly 1 size, optional
        y: InOut 1 1 ?,
        // parameter `b` has minimum size of 2 and maximum size of 10, required
        b: Out 2 10,
    }
}
```

Resulting struct would have fields with types from those:

`GetSingleOptional`

`GetSingle`

`GetMultipleOptional`

`GetMultiple`

`SetSingleOptional`

`SetSingle`

`SetMultipleOptional`

`SetMultiple`

`GetSetSingleOptional`

`GetSetSingle`

`GetSetMultipleOptional`

`GetSetMultiple`

*/

#[allow(clippy::too_many_lines)]
#[proc_macro]
pub fn ports(input: TokenStream) -> TokenStream {
    let PortsInput {
        attrs,
        visibility,
        struct_name,
        ports,
    } = parse_macro_input!(input as PortsInput);

    let mut report_blocks = vec![];
    let mut parse_blocks = vec![];
    let mut names = vec![];

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
            "In" => quote! { ::micrortu_sdk::IN },
            "Out" => quote! { ::micrortu_sdk::OUT },
            "InOut" => quote! { ::micrortu_sdk::IN_OUT },
            _ => {
                return syn::Error::new_spanned(port.mode, format!("Unknown port mode: {mode_str}"))
                    .to_compile_error()
                    .into()
            }
        };

        let to_nonzero_max_size = port.upper_bound.map_or(0, |m| m as u8);

        let name_str = name.to_string();
        let name_len = name_str.len() as u8;
        let name_offset = intern_static_string(&name_str);

        report_blocks.push(quote! {
            {

            #[cfg(target_arch = "wasm32")] {
                ::micrortu_sdk::BindingDefinition {
                    name_offset: #name_offset,
                    name_len: #name_len,
                    flags: #flags,
                    min_size: #min_size,
                    max_size: ::core::num::NonZeroU8::new(#to_nonzero_max_size),
                    direction: #direction,
                }
            }
            #[cfg(not(target_arch = "wasm32"))] {
                const NAME: &str = stringify!(#name);
                ::micrortu_sdk::NativeBindingDefinition::<'static> {
                    name: &NAME,
                    flags: #flags,
                    min_size: #min_size,
                    max_size: ::core::num::NonZeroU8::new(#to_nonzero_max_size),
                    direction: #direction,
                }
            }

            }
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

    let parse = quote! {
        fn parse(mut source: &'a mut [::micrortu_sdk::IEBuf], dirty: &'a mut u64)
            -> Result<Self, ::micrortu_sdk::ParseError>
        {
            let dirty = ::core::cell::Cell::from_mut(dirty);
            let mut cursor = 0;
            Ok(Self {
                #(#parse_blocks)*
            })
        }
    };

    let report = quote! {
        #[cfg(target_arch = "wasm32")]
        pub fn report() -> &'static [::micrortu_sdk::BindingDefinition] {
            static BINDINGS: &[::micrortu_sdk::BindingDefinition] = &[
                #(#report_blocks,)*
            ];
            BINDINGS
        }
        #[cfg(not(target_arch = "wasm32"))]
        pub fn report() -> &'static [::micrortu_sdk::NativeBindingDefinition<'static>] {
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
        }

        impl<'a> #struct_name<'a> {
            #parse
            #report
        }
    };

    expanded.into()
}

static STRINGS: Mutex<String> = Mutex::new(String::new());

#[proc_macro]
pub fn finalize(_: TokenStream) -> TokenStream {
    let final_string = STRINGS.lock().expect("poison").clone();
    let len = final_string.len();
    let bytes_array = final_string.as_bytes().iter().map(|&b| quote! { #b });
    let bytes_array = quote! { [ #(#bytes_array),* ] };
    let doc = format!(" Collected strings: {final_string:?}");

    let expanded = quote! {
        #[no_mangle]
        #[doc = #doc]
        static COLLECTED_STRINGS: [u8; #len] = #bytes_array;
    };

    expanded.into()
}

fn intern_static_string(s: &str) -> u16 {
    let mut strings = STRINGS.lock().expect("poison");
    let len = strings.len();
    strings.push_str(s);
    len.try_into().expect("too many strings interned")
}
