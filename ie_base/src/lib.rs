#![cfg_attr(not(test), no_std)]

pub mod iebuf;
pub mod small_ie;
pub mod generic_ie;

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
