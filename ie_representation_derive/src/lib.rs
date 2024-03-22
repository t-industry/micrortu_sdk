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

/// Finalize the build process.
/// That macro must be called at the end to embed metadata into the binary.
/// It creates a link section "metadata" with json data of all registered blocks
/// and exported symbol `COLLECTED_STRINGS` with all strings from the build.
/// `BindingDefinition`'s `name_offset` and `name_len` are referencing `COLLECTED_STRINGS`.
/// # Example
/// ```rust
/// finalize!();
/// ```
///
#[proc_macro]
pub fn finalize(_: TokenStream) -> TokenStream {
    finalize::finalize()
}

/// Register block.
/// That macro should be called for each block to register it.
/// # Example
/// ```rust
/// register_block!(BlockType, BlockName, factory, init, step);
/// ```
#[proc_macro]
pub fn register_block(input: TokenStream) -> TokenStream {
    register_block::register_block(input)
}

/// Derive macro for `Config` trait.
/// If block requires some configuration, it should be derived from `Config` trait.
/// It requires type to be `AsBytes` and `FromBytes`. Firmware will pass slice
/// of bytes and you should be able to call `from_bytes` method on it to get the
/// configuration. For C code you should be able to cast a pointer to your struct.
#[proc_macro_derive(Config, attributes(block_names))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    config::derive_config(input)
}

static STRINGS: Mutex<String> = Mutex::new(String::new());
static FINALIZED: AtomicBool = AtomicBool::new(false);
static BLOCK_CONFIGS: Mutex<BTreeMap<(String, String), BlockConf>> = Mutex::new(BTreeMap::new());
static BLOCKS: Mutex<BTreeMap<(String, String), Block>> = Mutex::new(BTreeMap::new());
static PARAMS: Mutex<BTreeMap<(String, String), Vec<micrortu_build_utils::Port>>> =
    Mutex::new(BTreeMap::new());
static PORTS: Mutex<BTreeMap<(String, String), Vec<micrortu_build_utils::Port>>> =
    Mutex::new(BTreeMap::new());

/**
# Macros for generating parser of arguments block requires.

## Example

```rust
ports! {
    #[block_names(block_name1, block_name2)]
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
#[proc_macro]
pub fn ports(input: TokenStream) -> TokenStream {
    bindings::bindings(input, true)
}

/**
# Macros for generating parser of arguments block requires.

## Example

```rust
params! {
    #[block_names(block_name1, block_name2)]
    pub struct Params {
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
