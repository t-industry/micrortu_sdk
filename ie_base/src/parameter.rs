use bitfield::{bitfield_bitrange, bitfield_fields};
use const_default::ConstDefault;
use int_enum::IntEnum;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[cfg(feature = "rkyv")]
use {
    bytecheck::CheckBytes,
    rkyv::{Archive, Portable, Serialize},
};

#[repr(C, packed)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(CheckBytes))]
/// TI112, `P_ME_NC_1`, Parameter of measured values, short floating point number
pub struct P_ME_NC_1 {
    pub value: f32,
    pub qpm: QPM,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
pub struct QPM(pub u8);

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, IntEnum)]
pub enum KPA {
    Unused = 0,
    ThresholdValue = 1,
    SmoothingFactor = 2,
    LowLimitForTx = 3,
    HighLimitForTx = 4,
    //  5..31 = reserved for standard
    // 32..63 = reserved for custom use
}

crate::unsafe_resolve_as!(P_ME_NC_1, r1, struct, rkyv::rend::f32_le, value, QPM, qpm);

// this is a cbindgen-friendly way of generating bitfield accessors
// cbindgen can't eat bitfield! macro directly
bitfield_bitrange! { struct QPM(u8) }

impl QPM {
    bitfield_fields! {
        u8;
        pub pop, set_pop: 7;
        pub lpc, set_lpc: 6;
        pub kpa, set_kpa: 5,0;
    }
}
