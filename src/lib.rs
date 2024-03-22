/*!
# MicroRTU SDK

Provides utilities to create wasm blocks for MicroRTU.

## Example

This is a basic example of a block that adds two numbers.

```rust
use micrortu_sdk::{
    ie_base::{SmallIE, TryUpdateFrom}, params, ports, register_block, Shared, StepResult
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

pub fn factory(_: &mut Shared) -> Option<&'static mut Counter> {
    static COUTNER: StaticCell<Counter> = StaticCell::new();
    Some(COUTNER.init(Counter))
}

pub fn init(_: &mut Shared, _: &mut Counter) -> StepResult {
    0
}

pub fn step(shared: &mut Shared, _: &mut Counter) -> StepResult {
    let mut ports = Ports::parse(&mut shared.latched_ports[..], &mut shared.dirty_ports).unwrap();

    let value = f32::from(ports.count.get().unwrap());
    let mut ie = SmallIE::ti13(Default::default());
    ie.try_update_from(value + 1.0).unwrap();
    micrortu_sdk::info!("Counter: {} -> {}", value as i32, value as i32 + 1);
    ports.count.set(ie);

    0
}

register_block!(Counter, counter, factory, init, step);

```

## Layout

To define block `block_name`, that can be later referenced in MicroRTU
configuration, you need export 3 functions from your final wasm blob.

Every implementation must export `init` function with signature `() -> ()`.

### `factory_{block_name}`

`factory_{block_name}` is a function that will be called to produce a wasm block.
It's signature should be `(i32) -> i32` and code must ensure it follows Rust's
semantics of that signagure:

```ignore
for<'a> extern "C" fn(&'a mut Shared) -> Option<&'static mut BlockName>;
```

Where `BlockName` is your block's type.

### `init_{block_name}`

`init_{block_name}` is a function that will be called before `step`. It's
signature should be `(i32, i32) -> i32` and code must ensure it follows Rust's
semantics of that signagure:

```ignore
for<'a> extern "C" fn(&'a mut Shared, &'a mut BlockName) -> StepResult;
```

### `step_{block_name}`

`step_{block_name}` is a function that will be called to make a "step". It's
signature should be `(i32, i32) -> i32` and code must ensure it follows Rust's
semantics of that signature:

```ignore
for<'a> extern "C" fn(&'a mut Shared, &'a mut BlockName) -> StepResult;
```

*/

#![cfg_attr(not(test), no_std)]

#[no_mangle]
static mut SHARED: MaybeUninit<Shared> = MaybeUninit::zeroed();

pub mod bump_allocator;
mod getters_setters;
pub mod log;
pub mod trap_err;

use core::mem::MaybeUninit;

pub use getters_setters::*;
pub use ie_base;
/// Macros for generating parser of arguments block requires.
pub use ie_representation_derive::{ports, params, finalize, register_block, Config};

pub use bump_allocator::BumpAllocator;
pub use ie_base::IEBuf;
pub use wasm_global_shared_data::{
    BindingDefinition, Direction, NativeBindingDefinition, ParseError, Shared, StepResult, IN,
    IN_OUT, OUT, REQUIRED,
};


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

pub trait Config: zerocopy::FromBytes + zerocopy::AsBytes { }
