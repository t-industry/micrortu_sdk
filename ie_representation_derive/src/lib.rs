#![allow(clippy::redundant_closure_for_method_calls)]

use proc_macro::TokenStream;

mod bindings;
mod config;
mod finalize;
mod import;
mod register_block;
mod state;

/// Finalize the build process.
/// That macro must be called after all block registrations in the same crate
/// to embed metadata into the binary. It creates a link section "metadata"
/// with json data of all registered blocks, an exported symbol
/// `COLLECTED_STRINGS` with all port and param names, and the
/// `PORTS_*`/`PARAMS_*` binding definition statics, whose `name_offset` and
/// `name_len` reference `COLLECTED_STRINGS`.
///
/// Optionally takes the complete list of block names and errors unless it
/// matches the registered blocks exactly, which catches blocks that were
/// registered in another crate or after `finalize!`.
///
/// Like `export_blocks!`, it also exports the hidden replay macros that let
/// other crates register these blocks with `import_blocks!`.
/// # Example
/// ```rust
/// finalize!();
/// // or, with the expected block list checked:
/// finalize!(counter, gauge);
/// ```
///
#[proc_macro]
pub fn finalize(input: TokenStream) -> TokenStream {
    finalize::finalize(input)
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

/// Export the blocks of a library crate.
/// Performs the same collection and validation as `finalize!` and exports one
/// hidden replay macro per block, so a final crate can register them with
/// `import_blocks!`. Emits no symbols, so it is the right closing macro for
/// library crates. Optionally takes the complete block list, like `finalize!`.
/// # Example
/// ```rust
/// export_blocks!();
/// ```
#[proc_macro]
pub fn export_blocks(input: TokenStream) -> TokenStream {
    finalize::export_blocks(input)
}

/// Register blocks exported by other crates.
/// Each path names the exporting crate and the block. The block's metadata is
/// replayed into this crate's registry, so `finalize!` covers it; the block's
/// code and wasm symbols come from the exporting crate.
/// # Example
/// ```rust
/// import_blocks!(lib_a::pid, lib_b::twv_ctrl);
/// ```
#[proc_macro]
pub fn import_blocks(input: TokenStream) -> TokenStream {
    import::import_blocks(input)
}

/// Entry point for the replay macros exported by `export_blocks!` and
/// `finalize!`. Not for direct use.
#[doc(hidden)]
#[proc_macro]
pub fn __replay_meta(input: TokenStream) -> TokenStream {
    import::replay_meta(input)
}

/// Derive macro for `Config` trait.
///
/// If block requires some configuration, it should be derived from `Config` trait.
/// It requires type to be `AsBytes` and `FromBytes`. Firmware will pass slice
/// of bytes and you should be able to call `from_bytes` method on it to get the
/// configuration. For C code you should be able to cast a pointer to your struct.
///
/// ## Attributes
///
/// - `block_names`: the list of blocks with that configuration.
/// - `required`: whether user is required to provide this conf (true by default).
#[proc_macro_derive(Config, attributes(block_names, required))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    config::derive_config(input)
}

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
