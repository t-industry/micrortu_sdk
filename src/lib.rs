/*!
# MicroRTU SDK

Provides utilities to create wasm blocks for MicroRTU.

## Example

This is a basic example of a block that adds two numbers.

```rust
use micrortu_sdk::{
    ports,
    Shared,
    StepResult,
    BindingDefinition,
    ie_base::{SmallIE, TryUpdateFrom},
};
use static_cell::StaticCell;

// Macro invocation to generate ports struct.
ports! {
    pub struct Ports {
        a: In 1 1,
        b: In 1 1,
        y: Out 1 1,
    }
}

// Block's internal state. Nothing in this case.
struct AddBlock;

// Exporting factory function to create a block. Name should be `factory_{block_name}`.
#[no_mangle]
pub extern "C" fn factory_add(shared: &mut Shared) -> Option<&'static mut AddBlock> {
    // Easy way to store single static value and get `&'static mut` reference to it.
    static ADD: StaticCell<AddBlock> = StaticCell::new();

    // Wasm block should report what ports it uses, so MicroRTU can connect them.
    let ports: &[BindingDefinition] = Ports::report(); // macro-generated function
    shared.ports[..ports.len()].copy_from_slice(ports); // copy to shared memory
    shared.used_ports_len = ports.len() as u32; // report how many ports we use
    shared.used_params_len = 0; // we are not using any parameters

    ADD.try_init(AddBlock)
}

#[no_mangle]
pub extern "C" fn init_add(_: &mut Shared, _: &mut AddBlock) -> StepResult {
    // We don't need to init anything in this case.
    // Usually we might set up some internal state here - `&mut AddBlock` is
    // passed here by MicroRTU. It has lifetime narrowed to the function call,
    // so you are only allowed access to it here. Rust will ensure you don't
    // keep any references to it after this function returns.
    0
}

#[no_mangle]
pub extern "C" fn step_add(shared: &mut Shared, _: &mut AddBlock) -> StepResult {
    // Parse ports from shared memory. `Ports::parse` can return `ParseError` in
    // case of failure. Read its documentation for more information.
    let mut ports = Ports::parse(&mut shared.latched_ports[..], &mut shared.dirty_ports).unwrap();
    // `ports.a` and `ports.b` were generated to have type `GetSingle`. Look at
    // for more information, like available methods and errors it can return.
    let a: SmallIE = ports.a.get().unwrap();
    let b: SmallIE = ports.b.get().unwrap();

    // We are creating new information element, that will be our result.
    let mut y = SmallIE::TI50(Default::default());
    // We are trying to update it from sum of `a` and `b`.
    y.try_update_from(f32::from(a) + f32::from(b)).unwrap();
    // `ports.y` was generated to have type `SetSingle`. Look at its documentation
    // for more information, like available methods and errors it can return.
    ports.y.set(y);

    0
}

```

## Layout

To define block `block_name`, that can be later referenced in MicroRTU
configuration, you need export 3 functions from your final wasm blob.

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
static mut SHARED: Shared = Shared::new();

mod getters_setters;
pub mod trap_err;
mod log;

pub use getters_setters::*;
pub use ie_base;
/// Macros for generating parser of arguments block requires.
pub use ie_representation_derive::ports;

pub use ie_base::IEBuf;
pub use wasm_global_shared_data::{
    BindingDefinition, Direction, ParseError, Shared, StepResult, IN, IN_OUT, OUT, REQUIRED,
};

#[no_mangle]
extern "C" fn init() {
    ::log::set_logger(&log::LOGGER).unwrap();
}

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
