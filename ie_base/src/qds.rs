use bitfield::{Bit, BitMut, BitRange, BitRangeMut};
use const_default::ConstDefault;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[cfg(feature = "rkyv")]
use {
    bytecheck::CheckBytes,
    rkyv::{Archive, Portable, Serialize},
};

#[derive(Debug, Clone, Copy, Default, ConstDefault)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)]
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
#[repr(transparent)]
pub struct RawQualityDescriptor(pub u8);

pub trait QualityDescriptorHolder {
    fn qds_raw(&self) -> RawQualityDescriptor;
    fn mut_qds_raw(&mut self) -> &mut RawQualityDescriptor;
    fn has_ov(&self) -> bool {
        false
    }
}

pub trait QualityDescriptor {
    fn ov(&self) -> bool;
    fn set_ov(&mut self, value: bool);
    fn bl(&self) -> bool;
    fn set_bl(&mut self, value: bool);
    fn sb(&self) -> bool;
    fn set_sb(&mut self, value: bool);
    fn nt(&self) -> bool;
    fn set_nt(&mut self, value: bool);
    fn iv(&self) -> bool;
    fn set_iv(&mut self, value: bool);

    fn is_bad(&self) -> bool {
        self.ov() || self.bl() || self.sb() || self.nt() || self.iv()
    }

    fn is_good(&self) -> bool {
        !self.is_bad()
    }

    fn update_from(&mut self, src: &dyn QualityDescriptor) {
        self.set_ov(src.ov());
        self.set_bl(src.bl());
        self.set_sb(src.sb());
        self.set_nt(src.nt());
        self.set_iv(src.iv());
    }
}

impl RawQualityDescriptor {
    pub const INVALID: Self = Self(0x80);
    pub const NONTOPICAL: Self = Self(0x40);
    pub const BAD: Self = Self(Self::INVALID.0 | Self::NONTOPICAL.0);
}

impl From<u8> for RawQualityDescriptor {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl BitRange<u8> for RawQualityDescriptor {
    fn bit_range(&self, m: usize, l: usize) -> u8 {
        self.0.bit_range(m, l)
    }
}

impl BitRangeMut<u8> for RawQualityDescriptor {
    fn set_bit_range(&mut self, m: usize, l: usize, value: u8) {
        self.0.set_bit_range(m, l, value);
    }
}

impl PartialOrd for RawQualityDescriptor {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        // todo(ddytopia): ask is it okay or not
        let mask = 0b1111_0000; // ov ?
        Some((self.0 & mask).cmp(&(other.0 & mask)).reverse())
    }
}

impl PartialEq for RawQualityDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for RawQualityDescriptor {}

#[macro_export]
macro_rules! impl_qds_for {
    ($T:ty) => {
        impl $crate::qds::QualityDescriptorHolder for $T {
            fn qds_raw(&self) -> $crate::qds::RawQualityDescriptor {
                self.raw
            }
            fn mut_qds_raw(&mut self) -> &mut $crate::qds::RawQualityDescriptor {
                &mut self.raw
            }
        }
    };
}

impl<T: QualityDescriptorHolder> QualityDescriptor for T {
    // OV bit not presnt in EmbeddedQualityDescriptorHolder
    fn ov(&self) -> bool {
        if self.has_ov() {
            self.qds_raw().bit(0)
        } else {
            false
        }
    }
    fn set_ov(&mut self, value: bool) {
        if self.has_ov() {
            self.mut_qds_raw().set_bit(0, value);
        }
    }

    fn bl(&self) -> bool {
        self.qds_raw().bit(4)
    }
    fn set_bl(&mut self, value: bool) {
        self.mut_qds_raw().set_bit(4, value);
    }

    fn sb(&self) -> bool {
        self.qds_raw().bit(5)
    }
    fn set_sb(&mut self, value: bool) {
        self.mut_qds_raw().set_bit(5, value);
    }

    fn nt(&self) -> bool {
        self.qds_raw().bit(6)
    }
    fn set_nt(&mut self, value: bool) {
        self.mut_qds_raw().set_bit(6, value);
    }

    fn iv(&self) -> bool {
        self.qds_raw().bit(7)
    }
    fn set_iv(&mut self, value: bool) {
        self.mut_qds_raw().set_bit(7, value);
    }
}

impl QualityDescriptorHolder for RawQualityDescriptor {
    fn qds_raw(&self) -> RawQualityDescriptor {
        *self
    }

    fn mut_qds_raw(&mut self) -> &mut RawQualityDescriptor {
        self
    }
}
