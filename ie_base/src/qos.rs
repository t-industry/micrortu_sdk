use bitfield::{Bit, BitMut, BitRange, BitRangeMut};
use const_default::ConstDefault;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[cfg(feature = "rkyv")]
use {
    bytecheck::CheckBytes,
    rkyv::{Archive, Portable, Serialize},
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum QL {
    Default = 0,
    Reserved(u8) = 1,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
pub struct RawQualifierOfSetpoint(pub u8);

pub trait QualifierOfSetpointHolder {
    fn qos_raw(&self) -> RawQualifierOfSetpoint;
    fn mut_qos_raw(&mut self) -> &mut RawQualifierOfSetpoint;
}

impl QualifierOfSetpointHolder for RawQualifierOfSetpoint {
    fn qos_raw(&self) -> RawQualifierOfSetpoint {
        *self
    }

    fn mut_qos_raw(&mut self) -> &mut RawQualifierOfSetpoint {
        self
    }
}

pub trait QualifierOfSetpoint {
    fn se(&self) -> bool;
    fn set_se(&mut self, value: bool);
    fn ql(&self) -> QL;
    fn set_ql(&mut self, value: QL);
    fn update_from(&mut self, src: &dyn QualifierOfSetpoint) {
        self.set_se(src.se());
        self.set_ql(src.ql());
    }
}

impl BitRange<u8> for RawQualifierOfSetpoint {
    fn bit_range(&self, m: usize, l: usize) -> u8 {
        self.0.bit_range(m, l)
    }
}

impl BitRangeMut<u8> for RawQualifierOfSetpoint {
    fn set_bit_range(&mut self, m: usize, l: usize, value: u8) {
        self.0.set_bit_range(m, l, value);
    }
}

impl PartialOrd for RawQualifierOfSetpoint {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let mask = 0b1111_1100;
        Some((self.0 & mask).cmp(&(other.0 & mask)).reverse())
    }
}

impl Eq for RawQualifierOfSetpoint {}

#[macro_export]
macro_rules! impl_qos_for {
    ($T:ty,$field:tt) => {
        impl $crate::qos::QualifierOfSetpointHolder for $T {
            fn qos_raw(&self) -> $crate::qos::RawQualifierOfSetpoint {
                self.$field
            }
            fn mut_qos_raw(&mut self) -> &mut $crate::qos::RawQualifierOfSetpoint {
                &mut self.$field
            }
        }
    };
    ($T:ty) => {
        impl $crate::qos::QualifierOfSetpointHolder for $T {
            fn qos_raw(&self) -> $crate::qos::RawQualifierOfSetpoint {
                self.raw
            }
            fn mut_qos_raw(&mut self) -> &mut $crate::qos::RawQualifierOfSetpoint {
                &mut self.raw
            }
        }
    };
}

impl<T: QualifierOfSetpointHolder> QualifierOfSetpoint for T {
    fn se(&self) -> bool {
        self.qos_raw().bit(7)
    }

    fn set_se(&mut self, value: bool) {
        self.mut_qos_raw().set_bit(7, value);
    }

    fn ql(&self) -> QL {
        match self.qos_raw().bit_range(6, 0) {
            0 => QL::Default,
            v => QL::Reserved(v),
        }
    }

    fn set_ql(&mut self, value: QL) {
        self.mut_qos_raw().set_bit_range(
            6,
            0,
            match value {
                QL::Default => 0,
                QL::Reserved(v) => v,
            },
        );
    }
}
