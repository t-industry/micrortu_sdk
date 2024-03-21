#![allow(clippy::redundant_closure_for_method_calls)]

use micrortu_build_utils::{Block, BlockConf};
use proc_macro::TokenStream;
use std::{
    collections::BTreeMap,
    sync::{atomic::AtomicBool, Mutex},
};

mod bindings;
mod config;
mod finalize;
mod register_block;

#[proc_macro]
pub fn finalize(_: TokenStream) -> TokenStream {
    finalize::finalize()
}

#[proc_macro]
pub fn register_block(input: TokenStream) -> TokenStream {
    register_block::register_block(input)
}

#[proc_macro_derive(Config, attributes(block_names))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    config::derive_config(input)
}

static STRINGS: Mutex<String> = Mutex::new(String::new());
static FINALIZED: AtomicBool = AtomicBool::new(false);
static BLOCK_CONFIGS: Mutex<BTreeMap<String, BlockConf>> = Mutex::new(BTreeMap::new());
static BLOCKS: Mutex<BTreeMap<(String, String), Block>> = Mutex::new(BTreeMap::new());
static PARAMS: Mutex<BTreeMap<(String, String), Vec<micrortu_build_utils::Port>>> = Mutex::new(BTreeMap::new());
static PORTS: Mutex<BTreeMap<(String, String), Vec<micrortu_build_utils::Port>>> = Mutex::new(BTreeMap::new());

/**
# Macros for generating parser of arguments block requires.

## Example

```rust
ports! {
    pub struct Ports(block_name) {
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
#[proc_macro]
pub fn ports(input: TokenStream) -> TokenStream {
    bindings::bindings(input, true)
}

/**
# Macros for generating parser of arguments block requires.

## Example

```rust
ports! {
    pub struct Ports(block_name) {
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
#[proc_macro]
pub fn params(input: TokenStream) -> TokenStream {
    bindings::bindings(input, false)
}
