use bitfield::{bitfield_bitrange, bitfield_fields, BitRange, BitRangeMut};
use const_default::ConstDefault;
use int_enum::IntEnum;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

use crate::{impl_qoc_for, qoc::RawQualifierOfCommand};

/// TI45, `C_SC_NA_1`, Single command
#[repr(C, packed)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
pub struct C_SC_NA_1 {
    pub value: SCO,
}

impl From<C_SC_NA_1> for bool {
    fn from(command: C_SC_NA_1) -> Self {
        command.value.scs()
    }
}

/// TI46, `C_DC_NA_1`, Double command
#[repr(C, packed)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
pub struct C_DC_NA_1 {
    pub dco: DCO,
}

/// TI48, `C_SE_NA_1`, Set-point command, normalized value
#[repr(C, packed)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
pub struct C_SE_NA_1 {
    pub dco: DCO,
}

#[derive(Debug)]
pub struct InvalidState;

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

/// TI50, `C_SE_NC_1`, Set-point command, short floating point number
#[repr(C, packed)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
pub struct C_SE_NC_1 {
    pub value: f32,
    pub qos: QOS,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
pub struct SCO(pub u8);

bitfield_bitrange! {struct SCO(u8)}
impl SCO {
    bitfield_fields! {
        u8;
        pub se, set_se: 7;
        pub qu, set_qu: 6, 2;
        pub scs, set_scs: 0;
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, IntEnum)]
pub enum DCS {
    NotPermitted1 = 0,
    Off = 1,
    On = 2,
    NotPermitted2 = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
pub struct DCO {
    pub raw: RawQualifierOfCommand,
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
impl_qoc_for!(DCO);

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
pub struct QOS(pub u8);

bitfield_bitrange! {struct QOS(u8)}
impl QOS {
    bitfield_fields! {
        u8;
        pub se, set_se: 7;
        pub qu, set_qu: 6, 0;
    }
}
