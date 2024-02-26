use bitfield::{Bit, BitMut, BitRange, BitRangeMut};
use const_default::ConstDefault;
use int_enum::IntEnum;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

use crate::{qds::QualityDescriptorHolder, impl_qds_for, RawQualityDescriptor};

/// TI1, `M_SP_NA_1`, Single-point information without time tag
#[repr(C, packed)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromZeroes, FromBytes)]
pub struct M_SP_NA_1 {
    pub value: SIQ,
}

/// TI3, `M_DP_NA_1`, Double-point information without time tag
#[repr(C, packed)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromZeroes, FromBytes)]
pub struct M_DP_NA_1 {
    pub value: DIQ,
}

/// TI13, `M_ME_NC_1`, Measured value, short floating point number
#[repr(C, packed)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromZeroes, FromBytes)]
pub struct M_ME_NE_1 {
    pub value: f32,
    pub qds: QDS,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromZeroes, FromBytes)]
pub struct SIQ {
    pub raw: RawQualityDescriptor
}
impl_qds_for!(SIQ);

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

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, IntEnum)]
pub enum DPI {
    Indeterminate1 = 0,
    Off = 1,
    On = 2,
    Indeterminate2 = 3,
}

impl From<bool> for DPI {
    fn from(value: bool) -> Self {
        if value {
            Self::On
        } else {
            Self::Off
        }
    }
}


#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromZeroes, FromBytes)]
pub struct DIQ {
    pub raw: RawQualityDescriptor,
}
impl_qds_for!(DIQ);

impl DIQ {
    #[must_use]
    pub fn dpi(&self) -> Option<DPI> {
        DPI::from_int(self.raw.bit_range(1, 0)).ok()
    }
    pub fn set_dpi(&mut self, value: DPI) -> &mut Self {
        self.raw.set_bit_range(1, 0, value.int_value());
        self
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, ConstDefault, PartialEq, AsBytes, FromZeroes, FromBytes)]
pub struct QDS {
    pub raw: RawQualityDescriptor,
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
