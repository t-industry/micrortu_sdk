#![doc = include_str!("../README.md")]
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

pub trait Config: zerocopy::FromBytes + zerocopy::IntoBytes {
    #[cfg(feature = "std")]
    fn config_schema() -> micrortu_build_utils::BlockConf;
}

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
