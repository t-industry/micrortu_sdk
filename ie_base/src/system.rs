use const_default::ConstDefault;
use int_enum::IntEnum;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[cfg(feature = "rkyv")]
use {
    bytecheck::CheckBytes,
    rkyv::{Archive, Portable, Serialize},
};

/// TI100, `C_IC_NA_1`, Interrogation command
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, Eq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)]
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
#[repr(transparent)]
pub struct C_IC_NA_1 {
    pub qoi: u8,
}

/// TI102, `C_RD_NA_1`, Read command
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, Eq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)]
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
#[repr(transparent)]
pub struct C_RD_NA_1 {}

/// TI104, `C_TS_NA_1`, Test command
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, Eq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)]
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
#[repr(C)]
pub struct C_TS_NA_1 {
    /// 0b10101010
    pub pat0: u8,
    /// 0b01010101
    pub pat1: u8,
}

/// Qualifier Of Interrogation
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, IntEnum)]
pub enum QOI {
    // 0 = not used
    // 1..19 = reserved for standard
    StationInterrogation = 20,
    Group1 = 21,
    Group2 = 22,
    Group3 = 23,
    Group4 = 24,
    Group5 = 25,
    Group6 = 26,
    Group7 = 27,
    Group8 = 28,
    Group9 = 29,
    Group10 = 30,
    Group11 = 31,
    Group12 = 32,
    Group13 = 33,
    Group14 = 34,
    Group15 = 35,
    Group16 = 36,
    // 37..63  = reserved for standard
    // 64..255 = reserved for custom use
}
