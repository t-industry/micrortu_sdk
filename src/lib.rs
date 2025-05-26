/*!
# MicroRTU SDK

Provides utilities to create wasm blocks for MicroRTU.

## Example

This is a basic example of a block that adds two numbers.

```rust
use micrortu_sdk::{
    params, ports, register_block, Shared, StepResult, FactoryInput
};
use static_cell::StaticCell;

pub struct Counter;

ports! {
    #[block_names(counter)]
    pub struct Ports {
      count: TI13 InOut 1 1,
    }
}
params! {
    #[block_names(counter)]
    pub struct Params {}
}

pub fn factory(_: &mut FactoryInput) -> Option<&'static mut Counter> {
    static COUTNER: StaticCell<Counter> = StaticCell::new();
    Some(COUTNER.init(Counter))
}

pub fn init(_: &mut Shared, _: &mut Counter) -> StepResult {
    0
}

pub fn step(shared: &mut Shared, _: &mut Counter) -> StepResult {
    let mut ports = Ports::parse(&mut shared.latched_ports[..]);

    ports.count.value += 1.;

    0
}

register_block!(Counter, counter, factory, init, step);

```

## WASM Binary Layout

To define block `block_name`, that can be later referenced in MicroRTU
configuration, you need export 3 functions from your final wasm blob.

### Required Exports

#### `init`

`init` function with signature `() -> ()`.

#### `SHARED`

`SHARED` symbol, aligned to 8 bytes, and must be valid
for reads and writes for at least 512 bytes.

#### `COLLECTED_STRINGS`

It should be `&[u8]`, which is a pointer to the start and length of the slice.
It should point to all names of the ports and params, concatenated.
`name_offset` and `name_len` are relative to this slice.

#### `factory_{block_name}`

`factory_{block_name}` is a function that will be called to produce a wasm block.
It's signature should be `(i32) -> i32` and code must ensure it follows Rust's
semantics of that signagure:

```ignore
for<'a> extern "C" fn(&'a FactoryInput) -> Option<&'static mut BlockName>;
```

Where `BlockName` is your block's type.

#### `init_{block_name}`

`init_{block_name}` is a function that will be called before `step`. It's
signature should be `(i32, i32) -> i32` and code must ensure it follows Rust's
semantics of that signagure:

```ignore
for<'a> extern "C" fn(&'a mut Shared, &'a mut BlockName) -> StepResult;
```

#### `step_{block_name}`

`step_{block_name}` is a function that will be called to make a "step". It's
signature should be `(i32, i32) -> i32` and code must ensure it follows Rust's
semantics of that signature:

```ignore
for<'a> extern "C" fn(&'a mut Shared, &'a mut BlockName) -> StepResult;
```

### `ports_{block_name}` and `params_{block_name}`

There also must be exports for ports and params of type `&[BindingDefinition]`,
which is [i32; 2] in memory - pointer to the start and length of the slice.



*/

#![cfg_attr(not(test), no_std)]

#[allow(dead_code)]
union Exported {
    shared: ManuallyDrop<Shared>,
    factory: ManuallyDrop<FactoryInput>,
}

#[no_mangle]
static mut SHARED: MaybeUninit<Exported> = MaybeUninit::zeroed();

pub mod bump_allocator;
mod getters_setters;
pub mod log;
pub mod trap_err;

use core::mem::{ManuallyDrop, MaybeUninit};

pub use getters_setters::*;
pub use ie_base;
/// Macros for generating parser of arguments block requires.
pub use ie_representation_derive::{finalize, params, ports, register_block, Config};
pub use wasm_global_shared_data;

pub use bump_allocator::BumpAllocator;
pub use ie_base::IEBuf;
pub use wasm_global_shared_data::{
    BindingDefinition, Direction, FactoryInput, NativeBindingDefinition, ParseError, Shared,
    StepResult, BINDINGS_BYTES_CAP, REQUIRED,
};
// these re-exports are used by `ie_representation_derive::register_block! macro`
pub use wasm_global_shared_data::{IN, IN_OUT, OUT};

pub use ufmt;

pub fn init_logger() {}

#[doc(hidden)]
pub fn wasm_unwrap<T>(v: Option<T>) -> T {
    match v {
        Some(v) => v,
        #[cfg(target_arch = "wasm32")]
        None => ::core::arch::wasm32::unreachable(),
        #[cfg(not(target_arch = "wasm32"))]
        None => unreachable!(),
    }
}

pub trait Config: zerocopy::FromBytes + zerocopy::AsBytes {}

pub trait BlockPorts<'a>: Sized {
    fn parse_fallible(source: &'a mut [u8]) -> Result<Self, ParseError>;
    fn parse(source: &'a mut [u8]) -> Self {
        match Self::parse_fallible(source) {
            Ok(binds) => binds,
            Err(_err) => {
                // error!("Failed to parse bindings: {:?}", err);
                #[cfg(target_arch = "wasm32")]
                {
                    ::core::arch::wasm32::unreachable()
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    panic!("Failed to parse bindings: {:?}", _err)
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn report() -> &'static [NativeBindingDefinition<'static>];
}
