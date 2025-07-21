# MicroRTU SDK

[![crates.io](https://img.shields.io/crates/v/micrortu_sdk.svg)](https://crates.io/crates/micrortu_sdk)
[![Documentation](https://docs.rs/micrortu_sdk/badge.svg)](https://docs.rs/micrortu_sdk)

Provides utilities to create wasm blocks for MicroRTU.

Documentation can be generated via `cargo doc --open` command.

## Example

This is a basic example of a block that adds two numbers.

```rust
use micrortu_sdk::{BlockPorts, FactoryInput, Shared, StepResult, params, ports, register_block};
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

pub fn factory(_: &FactoryInput) -> Option<&'static mut Counter> {
    static COUTNER: StaticCell<Counter> = StaticCell::new();
    Some(COUTNER.init(Counter))
}

pub fn init(_: &mut Shared, _: &mut Counter) -> StepResult {
    0
}

pub fn step(shared: &mut Shared, _: &mut Counter) -> StepResult {
    let ports = Ports::parse(&mut shared.latched_ports[..]);

    ports.count.value += 1.;

    0
}

register_block!(Counter, counter, factory, init, step);
```

## WASM Binary Layout for Non-Rust builds

If you don't want to use Rust and `micrortu_sdk` macros, you can still create a
wasm block for MicroRTU. The binary layout of the wasm blob must be as follows:

To define block `block_name`, that can be later referenced in MicroRTU
configuration, you need export 3 functions from your final wasm blob.

### Required Exports

#### `init`

`init` function with signature `() -> ()`.

#### `SHARED`

`SHARED` symbol, aligned to 8 bytes, and must be valid for reads and writes for
at least 512 bytes.

#### `COLLECTED_STRINGS`

It should be `&[u8]`, which is a pointer to the start and length of the slice.
It should point to all names of the ports and params, concatenated.
`name_offset` and `name_len` are relative to this slice.

#### `factory_{block_name}`

`factory_{block_name}` is a function that will be called to produce a wasm
block. It's signature should be `(i32) -> i32` and code must ensure it follows
Rust's semantics of that signagure:

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

## Environment Variables

`MICRORTU_BAIL_ON_DUPLICATES` - if set, compiler will check for duplicate
port/param definitions, confs and blocks themselves. If not set, last definition
would be used.
