use bitfield::{bitfield_bitrange, bitfield_fields};
use const_default::ConstDefault;
use int_enum::IntEnum;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

#[repr(C, packed)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
/// TI112, `P_ME_NC_1`, Parameter of measured values, short floating point number
pub struct P_ME_NC_1 {
    pub value: f32,
    pub qpm: QPM,
}

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

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
pub struct QPM(u8);

// this is a cbindgen-friendly way of generating bitfield accessors
// cbindgen can't eat bitfield! macro directly
bitfield_bitrange! {struct QPM(u8)}
impl QPM {
    bitfield_fields! {
        u8;
        pub pop, set_pop: 7;
        pub lpc, set_lpc: 6;
        pub kpa, set_kpa: 5,0;
    }
}
