use bitfield::{Bit, BitMut, BitRange, BitRangeMut};
use const_default::ConstDefault;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

#[cfg(feature = "rkyv")]
use {
    bytecheck::CheckBytes,
    rkyv::{Archive, Portable, Serialize},
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum QU {
    Default = 0,
    ShortPulse = 1,
    LongPulse = 2,
    PersistentOutput = 3,
    Reserved(u8) = 4,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(AsBytes, FromZeroes, FromBytes)] //
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
pub struct RawQualifierOfCommand(pub u8);

pub trait QualifierOfCommandHolder {
    fn qoc_raw(&self) -> RawQualifierOfCommand;
    fn mut_qoc_raw(&mut self) -> &mut RawQualifierOfCommand;
}

pub trait QualifierOfCommand {
    fn se(&self) -> bool;
    fn set_se(&mut self, value: bool) -> &mut Self;
    fn qu(&self) -> QU;
    fn set_qu(&mut self, value: QU) -> &mut Self;
}

impl BitRange<u8> for RawQualifierOfCommand {
    fn bit_range(&self, m: usize, l: usize) -> u8 {
        self.0.bit_range(m, l)
    }
}

impl BitRangeMut<u8> for RawQualifierOfCommand {
    fn set_bit_range(&mut self, m: usize, l: usize, value: u8) {
        self.0.set_bit_range(m, l, value);
    }
}

impl PartialOrd for RawQualifierOfCommand {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let mask = 0b1111_1100;
        Some((self.0 & mask).cmp(&(other.0 & mask)).reverse())
    }
}

impl Eq for RawQualifierOfCommand {}

#[macro_export]
macro_rules! impl_qoc_for {
    ($T:ty) => {
        impl $crate::qoc::QualifierOfCommandHolder for $T {
            fn qoc_raw(&self) -> $crate::qoc::RawQualifierOfCommand {
                self.raw
            }
            fn mut_qoc_raw(&mut self) -> &mut $crate::qoc::RawQualifierOfCommand {
                &mut self.raw
            }
        }
    };
}

impl<T: QualifierOfCommandHolder> QualifierOfCommand for T {
    fn se(&self) -> bool {
        self.qoc_raw().bit(7)
    }

    fn set_se(&mut self, value: bool) -> &mut Self {
        self.mut_qoc_raw().set_bit(7, value);
        self
    }

    fn qu(&self) -> QU {
        match self.qoc_raw().bit_range(6, 2) {
            0 => QU::Default,
            1 => QU::ShortPulse,
            2 => QU::LongPulse,
            3 => QU::PersistentOutput,
            v => QU::Reserved(v),
        }
    }

    fn set_qu(&mut self, value: QU) -> &mut Self {
        self.mut_qoc_raw().set_bit_range(
            6,
            2,
            match value {
                QU::Default => 0,
                QU::ShortPulse => 1,
                QU::LongPulse => 2,
                QU::PersistentOutput => 3,
                QU::Reserved(v) => v,
            },
        );
        self
    }
}
