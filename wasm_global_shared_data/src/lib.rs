#![no_std]

use core::num::NonZeroU16;
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

    pub dirty_params: u64,
    pub dirty_ports: u64,

    pub control_period_ms: u64,
}

pub const REQUIRED: u16 = 0x0001;

/// A binding definition.
///
/// Would be generated automatically by the `ports` macro.
///
/// For example, if block accepts a parameter `x` which is a non-empty vector
/// with maximum size of 10, the binding definition would be:
/// ```rust
/// BindingDefinition {
///    name: { let mut name = [0; 32]; name[..1].copy_from_slice(b"x"); name },
///    flags: REQUIRED,
///    min_size: 1,
///    max_size: Some(NonZeroU16::new(10).unwrap()),
///    direction: IN,
/// }
/// ```
#[repr(C)]
#[derive(Debug, AsBytes, FromZeroes, FromBytes, Clone, Copy)]
pub struct BindingDefinition {
    /// Utf-8 encoded string. If name is less then 32 bytes, it should be null-terminated.
    pub name: [u8; 32],
    pub flags: u16,
    pub min_size: u16,
    pub max_size: Option<NonZeroU16>,
    pub direction: Direction,
}

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
pub struct Direction(pub u16);

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
            name: [0; 32],
            flags: 0,
            min_size: 0,
            max_size: None,
            direction: IN,
        };

        Self {
            params: [bd; 64],
            ports: [bd; 64],
            used_params_len: 0,
            used_ports_len: 0,
            latched_params: [iebuf; 64],
            latched_ports: [iebuf; 64],
            dirty_params: 0,
            dirty_ports: 0,
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

impl BindingDefinition {
    /// Helper function to get the name of the binding.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        let len = self.name.iter().position(|&c| c == 0).unwrap_or(32);
        core::str::from_utf8(&self.name[..len]).ok()
    }
}
