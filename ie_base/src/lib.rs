#![cfg_attr(not(test), no_std)]

pub mod generic_ie;
pub mod iebuf;
pub mod small_ie;

pub mod address;
pub mod command;
pub mod measurement;
pub mod parameter;
pub mod qds;
pub mod qoc;
pub mod system;

pub mod conversion_impls;
pub mod query_impls;

pub use iebuf::*;
pub use small_ie::*;

pub use address::*;
pub use command::*;
pub use measurement::*;
pub use parameter::*;
pub use qds::*;
pub use system::*;

#[cfg(feature = "rkyv")]
mod rkyv_macros;

#[derive(Debug, Copy, Clone)]
pub struct IEConversionError;

pub trait TryUpdateFrom<T> {
    type Error;

    /// Try to update the value from the given value.
    ///
    /// # Errors
    ///
    /// Will return an error if the value is not valid for the type.
    fn try_update_from(&mut self, value: T) -> Result<(), Self::Error>;
}

impl core::error::Error for IEConversionError {}
impl core::fmt::Display for IEConversionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Invalid conversion")
    }
}
