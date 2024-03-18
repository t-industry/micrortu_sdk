#![no_std]

use core::num::NonZeroU8;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

pub use ie_base::IEBuf;

/// Shared data between the wasm module and the host.
#[repr(C)]
#[derive(AsBytes, FromZeroes, FromBytes)]
pub struct Shared {
    pub params: [BindingDefinition; 64],
    pub ports: [BindingDefinition; 64],
    pub used_params_len: u32,
    pub used_ports_len: u32,

    pub latched_params: [IEBuf; 64],
    pub latched_ports: [IEBuf; 64],

    pub dirty_params: [u8; 8],
    pub dirty_ports: [u8; 8],

    pub control_period_ms: u64,
}

pub const REQUIRED: u16 = 0x0001;

/// Erorrs that can occur while parsing genarated ports from `Shared`,
/// written by `MicroRTU`.
/// Indicates misconfiguration of `MicroRTU` or a bug in `ports!` macro or
/// `MicroRTU` firmware.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ParseError {
    NotTerminated,
    NotEnoughData,
    TooMuchData,
    InvalidData,
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
        let iebuf = IEBuf([0; core::mem::size_of::<IEBuf>()]);
        let bd = BindingDefinition {
            name_offset: 0,
            name_len: 0,
            flags: 0,
            min_size: 0,
            max_size: None,
            direction: Direction(0),
        };

        Self {
            params: [bd; 64],
            ports: [bd; 64],
            used_params_len: 0,
            used_ports_len: 0,
            latched_params: [iebuf; 64],
            latched_ports: [iebuf; 64],
            dirty_params: [0; 8],
            dirty_ports: [0; 8],
            control_period_ms: 0,
        }
    }

    pub fn set_ports(&mut self, ports: &[BindingDefinition]) {
        self.ports[..ports.len()].copy_from_slice(ports);
        self.used_ports_len = ports.len() as u32;
    }

    pub fn set_params(&mut self, params: &[BindingDefinition]) {
        self.params[..params.len()].copy_from_slice(params);
        self.used_params_len = params.len() as u32;
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
    pub flags: u16,
    pub min_size: u8,
    pub max_size: Option<NonZeroU8>,
    pub direction: Direction,
    pub name_len: u8,
}

/// A `BindingDefinition` for native (non-wasm) blocks.
#[derive(Debug, Clone, Copy)]
pub struct NativeBindingDefinition<'a> {
    pub name: &'a str,
    pub flags: u16,
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
