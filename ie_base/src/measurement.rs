#![allow(non_camel_case_types)]

use bitfield::{Bit, BitMut, BitRange, BitRangeMut};
use const_default::ConstDefault;
use int_enum::IntEnum;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[cfg(feature = "rkyv")]
use {
    bytecheck::CheckBytes,
    rkyv::{Archive, Portable, Serialize},
};

use crate::{impl_qds_for, qds::QualityDescriptorHolder, RawQualityDescriptor};

/// TI1, `M_SP_NA_1`, Single-point information without time tag
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct M_SP_NA_1 {
    pub value: SIQ,
}

/// TI3, `M_DP_NA_1`, Double-point information without time tag
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct M_DP_NA_1 {
    pub value: DIQ,
}

/// TI11, `M_ME_NB_1`, Measured value, scaled value
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct M_ME_NB_1 {
    pub value: i16,
    pub qds: QDS,
}

/// TI13, `M_ME_NC_1`, Measured value, short floating point number
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct M_ME_NE_1 {
    pub value: f32,
    pub qds: QDS,
}

/// TI136, Measured value, 32-bit unsigned integer
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct TI136 {
    pub value: u32,
    pub qds: QDS,
}

/// TI137, Measured value, 32-bit signed integer
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct TI137 {
    pub value: i32,
    pub qds: QDS,
}

/// TI138, Measured value, 64-bit unsigned integer
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct TI138 {
    pub value: u64,
    pub qds: QDS,
}

/// TI139, Measured value, 64-bit signed integer
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct TI139 {
    pub value: i64,
    pub qds: QDS,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
pub struct SIQ {
    pub raw: RawQualityDescriptor,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
pub struct QDS {
    pub raw: RawQualityDescriptor,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
pub struct DIQ {
    pub raw: RawQualityDescriptor,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, IntEnum)]
pub enum DPI {
    Indeterminate1 = 0,
    Off = 1,
    On = 2,
    Indeterminate2 = 3,
}

#[cfg(feature = "rkyv")]
mod impls {
    use super::*;
    use crate::unsafe_resolve_as;
    use rkyv::rend::{f32_le, i16_le, i32_le, i64_le, u32_le, u64_le};

    unsafe_resolve_as!(M_SP_NA_1, r1, struct, SIQ, value);
    unsafe_resolve_as!(M_DP_NA_1, r2, struct, DIQ, value);
    unsafe_resolve_as!(M_ME_NB_1, r3, struct, i16_le, value, QDS, qds);
    unsafe_resolve_as!(M_ME_NE_1, r4, struct, f32_le, value, QDS, qds);
    unsafe_resolve_as!(TI136, r5, struct, u32_le, value, QDS, qds);
    unsafe_resolve_as!(TI137, r6, struct, i32_le, value, QDS, qds);
    unsafe_resolve_as!(TI138, r7, struct, u64_le, value, QDS, qds);
    unsafe_resolve_as!(TI139, r8, struct, i64_le, value, QDS, qds);
}

impl_qds_for!(SIQ);
impl_qds_for!(DIQ);

impl SIQ {
    pub const BAD: Self = Self {
        raw: RawQualityDescriptor::BAD,
    };
    pub const INVALID: Self = Self {
        raw: RawQualityDescriptor::INVALID,
    };
    pub const NONTOPICAL: Self = Self {
        raw: RawQualityDescriptor::NONTOPICAL,
    };
}

impl DIQ {
    pub const BAD: Self = Self {
        raw: RawQualityDescriptor::BAD,
    };
    pub const INVALID: Self = Self {
        raw: RawQualityDescriptor::INVALID,
    };
    pub const NONTOPICAL: Self = Self {
        raw: RawQualityDescriptor::NONTOPICAL,
    };
}

impl QDS {
    pub const BAD: Self = Self {
        raw: RawQualityDescriptor(0xC0),
    };
    pub const INVALID: Self = Self {
        raw: RawQualityDescriptor::INVALID,
    };
    pub const NONTOPICAL: Self = Self {
        raw: RawQualityDescriptor::NONTOPICAL,
    };
}

impl SIQ {
    #[must_use]
    pub fn spi(&self) -> bool {
        self.raw.bit(0)
    }
    pub fn set_spi(&mut self, value: bool) -> &mut Self {
        self.raw.set_bit(0, value);
        self
    }
}

impl DIQ {
    #[must_use]
    pub fn dpi(&self) -> DPI {
        DPI::try_from(self.raw.bit_range(1, 0)).unwrap()
    }
    pub fn set_dpi(&mut self, value: DPI) -> &mut Self {
        self.raw.set_bit_range(1, 0, u8::from(value));
        self
    }
}

impl QualityDescriptorHolder for QDS {
    fn qds_raw(&self) -> RawQualityDescriptor {
        self.raw
    }
    fn mut_qds_raw(&mut self) -> &mut RawQualityDescriptor {
        &mut self.raw
    }
    fn has_ov(&self) -> bool {
        true
    }
}
