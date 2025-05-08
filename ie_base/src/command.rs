#![allow(non_camel_case_types)]

use bitfield::{bitfield_bitrange, bitfield_fields, BitRange, BitRangeMut};
use const_default::ConstDefault;
use int_enum::IntEnum;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

#[cfg(feature = "rkyv")]
use {
    bytecheck::CheckBytes,
    rkyv::{Archive, Portable, Serialize},
};

use crate::{impl_qoc_for, qoc::RawQualifierOfCommand};

/// TI45, `C_SC_NA_1`, Single command
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct C_SC_NA_1 {
    pub value: SCO,
}

/// TI46, `C_DC_NA_1`, Double command
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct C_DC_NA_1 {
    pub dco: DCO,
}

/// TI48, `C_SE_NA_1`, Set-point command, normalized value
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct C_SE_NA_1 {
    pub dco: DCO,
}

/// TI49, `C_SE_NB_1`, Set-point command, normalized value
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct C_SE_NB_1 {
    pub value: i16,
    pub qos: QOS,
}

/// TI50, `C_SE_NC_1`, Set-point command, short floating point number
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct C_SE_NC_1 {
    pub value: f32,
    pub qos: QOS,
}

/// TI200, Set-point command, 32-bit unsigned integer
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromZeroes, FromBytes)]
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct TI200 {
    pub value: u32,
    pub qos: QOS,
}

/// TI201, Set-point command, 32-bit signed integer
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromZeroes, FromBytes)]
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct TI201 {
    pub value: i32,
    pub qos: QOS,
}

/// TI202, Set-point command, 64-bit unsigned integer
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromZeroes, FromBytes)]
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct TI202 {
    pub value: u64,
    pub qos: QOS,
}

/// TI203, Set-point command, 64-bit signed integer
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromZeroes, FromBytes)]
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
pub struct TI203 {
    pub value: i64,
    pub qos: QOS,
}

#[derive(Debug)]
pub struct InvalidState;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(AsBytes, FromZeroes, FromBytes)] //
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
pub struct SCO(pub u8);

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, IntEnum)]
pub enum DCS {
    NotPermitted1 = 0,
    Off = 1,
    On = 2,
    NotPermitted2 = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(AsBytes, FromZeroes, FromBytes)] //
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
pub struct DCO {
    pub raw: RawQualifierOfCommand,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(AsBytes, FromZeroes, FromBytes)] //
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
pub struct QOS(pub u8);

bitfield_bitrange! { struct SCO(u8) }
bitfield_bitrange! { struct QOS(u8) }

#[cfg(feature = "rkyv")]
mod impls {
    use super::*;
    use crate::unsafe_resolve_as;
    use rkyv::rend::{f32_le, i16_le, i32_le, i64_le, u32_le, u64_le};

    unsafe_resolve_as!(C_SC_NA_1, r1, struct, SCO, value);
    unsafe_resolve_as!(C_DC_NA_1, r2, struct, DCO, dco);
    unsafe_resolve_as!(C_SE_NA_1, r3, struct, DCO, dco);
    unsafe_resolve_as!(C_SE_NB_1, r4, struct, i16_le, value, QOS, qos);
    unsafe_resolve_as!(C_SE_NC_1, r5, struct, f32_le, value, QOS, qos);
    unsafe_resolve_as!(TI200, r6, struct, u32_le, value, QOS, qos);
    unsafe_resolve_as!(TI201, r7, struct, i32_le, value, QOS, qos);
    unsafe_resolve_as!(TI202, r8, struct, u64_le, value, QOS, qos);
    unsafe_resolve_as!(TI203, r9, struct, i64_le, value, QOS, qos);
}

impl_qoc_for!(DCO);

impl TryFrom<C_DC_NA_1> for bool {
    type Error = InvalidState;

    fn try_from(value: C_DC_NA_1) -> Result<Self, Self::Error> {
        match value.dco.dcs() {
            DCS::Off => Ok(false),
            DCS::On => Ok(true),
            DCS::NotPermitted1 | DCS::NotPermitted2 => Err(InvalidState),
        }
    }
}

impl From<C_SC_NA_1> for bool {
    fn from(command: C_SC_NA_1) -> Self {
        command.value.scs()
    }
}

impl SCO {
    bitfield_fields! {
        u8;
        pub se, set_se: 7;
        pub qu, set_qu: 6, 2;
        pub scs, set_scs: 0;
    }
}

impl DCO {
    #[must_use]
    pub fn dcs(&self) -> DCS {
        DCS::try_from(self.raw.bit_range(1, 0)).unwrap()
    }
    pub fn set_dcs(&mut self, value: DCS) -> &mut Self {
        self.raw.set_bit_range(1, 0, u8::from(value));
        self
    }
}

impl QOS {
    bitfield_fields! {
        u8;
        pub se, set_se: 7;
        pub qu, set_qu: 6, 0;
    }
}
