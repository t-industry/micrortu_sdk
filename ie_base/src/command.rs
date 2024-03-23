use bitfield::{bitfield_bitrange, bitfield_fields};
use const_default::ConstDefault;
use zerocopy::{AsBytes, FromZeroes, FromBytes};

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
pub struct SCO(u8);

bitfield_bitrange! {struct SCO(u8)}
impl SCO {
    bitfield_fields! {
        u8;
        pub se, set_se: 7;
        pub qu, set_qu: 6, 2;
        pub scs, set_scs: 0;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromBytes, FromZeroes)]
pub struct QOS(u8);

bitfield_bitrange! {struct QOS(u8)}
impl QOS {
    bitfield_fields! {
        u8;
        pub se, set_se: 7;
        pub qu, set_qu: 6, 0;
    }
}
