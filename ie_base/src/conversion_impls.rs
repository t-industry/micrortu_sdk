use int_enum::{IntEnum, IntEnumError};
use num::{Num, ToPrimitive};

use crate::qds::QualityDescriptorHolder;

use crate::{measurement::DPI, IEConversionError, SmallIE, TryUpdateFrom};

impl<T: IntEnum> From<IntEnumError<T>> for IEConversionError {
    fn from(_value: IntEnumError<T>) -> Self {
        Self
    }
}

impl<T: Num + ToPrimitive> TryUpdateFrom<T> for SmallIE {
    type Error = IEConversionError;

    fn try_update_from(&mut self, value: T) -> Result<(), Self::Error> {
        match self {
            Self::TI1(ie) => {
                ie.value.set_spi(!value.is_zero());
            }
            Self::TI3(ie) => {
                ie.value
                    .set_dpi(DPI::from_int(value.to_u8().ok_or(IEConversionError)?)?);
            }
            Self::TI13(ie) => {
                ie.value = value.to_f32().ok_or(IEConversionError)?;
            }
            Self::TI45(ie) => {
                ie.value.set_scs(!value.is_zero());
            }
            Self::TI50(ie) => {
                ie.value = value.to_f32().ok_or(IEConversionError)?;
            }
            Self::TI112(ie) => {
                ie.value = value.to_f32().ok_or(IEConversionError)?;
            }
        }

        Ok(())
    }
}

impl From<SmallIE> for f32 {
    fn from(value: SmallIE) -> Self {
        match value {
            SmallIE::TI1(ie) => {
                if ie.value.spi() {
                    1.0
                } else {
                    0.0
                }
            }
            SmallIE::TI3(ie) => (ie.value.dpi() as u8).to_f32().unwrap_or_default(),
            SmallIE::TI13(ie) => ie.value,
            SmallIE::TI45(ie) => {
                if ie.value.scs() {
                    1.0
                } else {
                    0.0
                }
            }
            SmallIE::TI50(ie) => ie.value,
            SmallIE::TI112(ie) => ie.value,
        }
    }
}

impl TryUpdateFrom<Self> for SmallIE {
    type Error = IEConversionError;

    fn try_update_from(&mut self, value: Self) -> Result<(), Self::Error> {
        match (self, value) {
            (Self::TI1(ie), Self::TI1(ie_src)) => *ie = ie_src,
            (Self::TI1(ie), Self::TI3(ie_src)) => {
                *ie.value.mut_qds_raw() = ie_src.value.qds_raw();
                ie.value.set_spi(ie_src.value.dpi() == DPI::On);
            }
            (Self::TI3(ie), Self::TI1(ie_src)) => {
                *ie.value.mut_qds_raw() = ie_src.value.qds_raw();
                ie.value.set_dpi(ie_src.value.spi().into());
            }
            (Self::TI3(ie), Self::TI3(ie_src)) => *ie = ie_src,
            (Self::TI13(ie), Self::TI13(ie_src)) => *ie = ie_src,
            (Self::TI45(ie), Self::TI45(ie_src)) => *ie = ie_src,
            (Self::TI50(ie), Self::TI50(ie_src)) => *ie = ie_src,
            (Self::TI112(ie), Self::TI112(ie_src)) => *ie = ie_src,
            _ => return Err(IEConversionError),
        }

        Ok(())
    }
}

impl TryFrom<SmallIE> for bool {
    type Error = IEConversionError;

    fn try_from(value: SmallIE) -> Result<Self, Self::Error> {
        match value {
            SmallIE::TI1(ie) => Ok(ie.value.spi()),
            SmallIE::TI45(ie) => Ok(ie.value.scs()),
            SmallIE::TI3(ie) => match ie.value.dpi() {
                DPI::Off => Ok(false),
                DPI::On => Ok(true),
                _ => Err(IEConversionError),
            },
            _ => Err(IEConversionError {}),
        }
    }
}
