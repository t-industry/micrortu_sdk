pub mod command;
pub mod iebuf;
pub mod impls;
pub mod measurement;
pub mod parameter;
pub mod system;
pub mod type_desc;

pub use iebuf::IEBuf;

use crate::qds::QualityDescriptor;

use self::{
    measurement::{QDS, SIQ},
    system::QOI,
    type_desc::{TypeDesc, TYPE_DESC},
};

use super::ie::{
    command::{C_SC_NA_1, C_SE_NC_1},
    measurement::{M_DP_NA_1, M_ME_NE_1, M_SP_NA_1},
    parameter::P_ME_NC_1,
    system::{C_IC_NA_1, C_RD_NA_1, C_TS_NA_1},
};

use derive_more::Display;
use zerocopy::AsBytes;

#[derive(Debug, Copy, Clone)]
pub struct IEConversionError;

pub trait TryUpdateFrom<T> {
    type Error;

    /// Try to update the value from the given value.
    ///
    /// # Errors
    ///
    /// Will return an error if the value is not valid for the type.
    fn try_update_from(&mut self, value: T) -> Result<(), Self::Error>;
}

#[derive(Debug, Display)]
pub enum Error {
    CopyToError,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IE {
    TI1(M_SP_NA_1),
    TI3(M_DP_NA_1),
    TI13(M_ME_NE_1),
    TI45(C_SC_NA_1),
    TI50(C_SE_NC_1),
    TI100(C_IC_NA_1),
    TI102(C_RD_NA_1),
    TI104(C_TS_NA_1),
    TI112(P_ME_NC_1),
}

impl IE {
    #[must_use]
    pub const fn typecode(&self) -> u8 {
        match self {
            Self::TI1(_) => 1,
            Self::TI3(_) => 3,
            Self::TI13(_) => 13,
            Self::TI45(_) => 45,
            Self::TI50(_) => 50,
            Self::TI100(_) => 100,
            Self::TI102(_) => 102,
            Self::TI104(_) => 104,
            Self::TI112(_) => 112,
        }
    }

    #[must_use]
    pub fn default_for_typecode(typecode: u8) -> Option<Self> {
        Some(match typecode {
            1 => Self::TI1(Default::default()),
            3 => Self::TI3(Default::default()),
            13 => Self::TI13(Default::default()),
            45 => Self::TI45(Default::default()),
            50 => Self::TI50(Default::default()),
            100 => Self::TI100(Default::default()),
            102 => Self::TI102(Default::default()),
            104 => Self::TI104(Default::default()),
            112 => Self::TI112(Default::default()),
            _ => return None,
        })
    }

    fn bytes(&self) -> &[u8] {
        match self {
            Self::TI1(it) => it.as_bytes(),
            Self::TI3(it) => it.as_bytes(),
            Self::TI13(it) => it.as_bytes(),
            Self::TI45(it) => it.as_bytes(),
            Self::TI50(it) => it.as_bytes(),
            Self::TI100(it) => it.as_bytes(),
            Self::TI102(it) => it.as_bytes(),
            Self::TI104(it) => it.as_bytes(),
            Self::TI112(it) => it.as_bytes(),
        }
    }

    pub fn type_desc(&self) -> &'static TypeDesc {
        let tc = self.typecode();
        TYPE_DESC.iter().find(|td| td.ti == tc).unwrap()
    }

    #[must_use]
    pub fn current_size(&self) -> usize {
        self.bytes().len()
    }

    pub fn copy_to(&self, dest: &mut [u8]) -> Result<usize, Error> {
        let dest = dest
            .get_mut(..self.bytes().len())
            .ok_or(Error::CopyToError)?;

        dest.copy_from_slice(self.bytes());

        Ok(dest.len())
    }

    #[must_use]
    pub fn to_monitor_direction(&self) -> Option<Self> {
        if (self.type_desc().cot_mon)(QOI::StationInterrogation as u8) {
            return Some(*self);
        }

        match self {
            Self::TI45(v) => Some(Self::TI1(M_SP_NA_1 {
                value: *SIQ::default().set_spi(v.value.scs()),
            })),
            Self::TI50(v) => Some(Self::TI13(M_ME_NE_1 {
                value: v.value,
                qds: QDS::default(),
            })),
            _ => None,
        }
    }

    #[must_use]
    pub fn try_get_qds(&self) -> Option<&dyn QualityDescriptor> {
        match self {
            Self::TI1(ie) => Some(&ie.value),
            Self::TI3(ie) => Some(&ie.value),
            Self::TI13(ie) => Some(&ie.qds),
            Self::TI45(_) | Self::TI50(_) | Self::TI112(_) => None,
            _ => unimplemented!(),
        }
    }

    pub fn try_get_qds_mut(&mut self) -> Option<&mut dyn QualityDescriptor> {
        match self {
            Self::TI1(ie) => Some(&mut ie.value),
            Self::TI3(ie) => Some(&mut ie.value),
            Self::TI13(ie) => Some(&mut ie.qds),
            Self::TI45(_) | Self::TI50(_) | Self::TI112(_) => None,
            _ => unimplemented!(),
        }
    }
}
