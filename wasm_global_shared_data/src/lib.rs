#![no_std]

use core::num::NonZeroU8;
use ufmt::derive::uDebug;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

pub use ie_base::IEBuf;

#[repr(C, align(8))]
#[derive(AsBytes, FromZeroes, FromBytes, Debug)]
pub struct Config(pub [u8; 504]);

/// Shared data between the wasm module and the host.
#[repr(C, align(8))]
#[derive(AsBytes, FromZeroes, FromBytes, Debug)]
pub struct Shared {
    pub latched_params: [u8; 256],
    pub latched_ports: [u8; 256],
}

/// Shared data between the wasm module and the host.
#[repr(C, align(8))]
#[derive(AsBytes, FromZeroes, FromBytes, Debug)]
pub struct FactoryInput {
    pub control_period_ms: u32,
    pub config_len: u32,
    pub config: Config
}

pub const REQUIRED: u8 = 0x0001;

/// Erorrs that can occur while parsing genarated ports from `Shared`,
/// written by `MicroRTU`.
/// Indicates misconfiguration of `MicroRTU` or a bug in `ports!` macro or
/// `MicroRTU` firmware.
#[repr(u8)]
#[derive(Debug, Clone, Copy, uDebug)]
pub enum ParseError {
    NotTerminated,
    NotEnoughData,
    TooMuchData,
    InvalidData,
    BadHeader,
    MultiplePointsForSingular,
}

/// A direction of a binding.
/// Meaningful values are `IN`, `OUT`, `IN_OUT`.
/// All other values are invalid, but safe.
#[repr(C)]
#[derive(Debug, AsBytes, FromZeroes, FromBytes, Clone, Copy)]
pub struct Direction(pub u8);

/// The result of a step. `0` means success, anything else is an error.
/// Implementation could also trap, but it's not recommended.
/// Any error would be logged.
pub type StepResult = i32;

/// Represents an input binding.
pub const IN: Direction = Direction(0);
/// Represents an output binding.
pub const OUT: Direction = Direction(1);
/// Represents an input-output binding.
pub const IN_OUT: Direction = Direction(2);

impl Default for Shared {
    fn default() -> Self {
        Self::new()
    }
}

impl Shared {
    /// Creates a new `Shared` instance with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            latched_params: [0; 256],
            latched_ports: [0; 256],
        }
    }

}

/// A binding definition.
///
/// Would be generated automatically by the `ports` macro.
///
/// For example, if block accepts a parameter `x` which is a non-empty vector
/// with maximum size of 10, the binding definition would be:
/// ```rust
/// use micrortu_wasm_global_shared_data::{IN, REQUIRED, BindingDefinition};
/// use core::num::NonZeroU8;
///
/// let def = BindingDefinition {
///    name_offset: 0,
///    name_len: 0,
///    flags: REQUIRED,
///    min_size: 1,
///    max_size: Some(NonZeroU8::new(10).unwrap()),
///    direction: IN,
/// };
/// ```
#[repr(C)]
#[derive(Debug, AsBytes, FromZeroes, FromBytes, Clone, Copy)]
pub struct BindingDefinition {
    pub name_offset: u16,
    pub flags: u8,
    pub typ: u8,
    pub min_size: u8,
    pub max_size: Option<NonZeroU8>,
    pub direction: Direction,
    pub name_len: u8,
}

/// A `BindingDefinition` for native (non-wasm) blocks.
#[derive(Debug, Clone, Copy)]
pub struct NativeBindingDefinition<'a> {
    pub name: &'a str,
    pub flags: u8,
    pub typ: u8,
    pub min_size: u8,
    pub max_size: Option<NonZeroU8>,
    pub direction: Direction,
}

impl BindingDefinition {
    // Returns the name of the binding.
    // # Arguments
    // `collected_names` - a slice that starts at the beginning of the collected names.
    //                     It is allowed to have extra data after the names.
    #[must_use]
    pub fn name<'a>(&self, collected_names: &'a [u8]) -> Option<&'a str> {
        let offset = self.name_offset as usize;
        let len = self.name_len as usize;
        core::str::from_utf8(collected_names.get(offset..)?.get(..len)?).ok()
    }
}

impl BindingDefinition {
    #[must_use]
    pub fn into_native(self, collected_names: &[u8]) -> Option<NativeBindingDefinition> {
        Some(NativeBindingDefinition {
            name: self.name(collected_names)?,
            typ: self.typ,
            flags: self.flags,
            min_size: self.min_size,
            max_size: self.max_size,
            direction: self.direction,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::Shared;
    use zerocopy::{AsBytes, FromZeroes};

    #[test]
    fn assert_shared_default_zeroed() {
        assert_eq!(Shared::new().as_bytes(), Shared::new_zeroed().as_bytes());
        assert_eq!(Shared::default().as_bytes(), Shared::new_zeroed().as_bytes());
    }
}
