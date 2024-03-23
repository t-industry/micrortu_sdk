#![cfg_attr(not(test), no_std)]

pub mod small_ie;
pub mod iebuf;

pub mod command;
pub mod measurement;
pub mod parameter;
pub mod system;
pub mod qds;
pub mod address;

pub mod conversion_impls;
pub mod query_impls;

pub use small_ie::*;
pub use iebuf::*;

pub use command::*;
pub use measurement::*;
pub use parameter::*;
pub use system::*;
pub use qds::*;
pub use address::*;

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
