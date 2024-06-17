use num::{Float, Num, ToPrimitive};

use crate::{
    measurement::DPI, qds::QualityDescriptorHolder, IEConversionError, SmallIE, TryUpdateFrom,
    C_SC_NA_1, C_SE_NC_1, DIQ, M_DP_NA_1, M_ME_NB_1, M_ME_NE_1, M_SP_NA_1, P_ME_NC_1, SIQ, TI1,
    TI11, TI112, TI13, TI136, TI137, TI138, TI139, TI200, TI201, TI202, TI203, TI3, TI45, TI50,
};

impl<T: Num + ToPrimitive> TryUpdateFrom<T> for SmallIE {
    type Error = IEConversionError;

    fn try_update_from(&mut self, value: T) -> Result<(), Self::Error> {
        match self {
            Self::TI1(ie) => _ = ie.value.set_spi(!value.is_zero()),
            Self::TI3(ie) => {
                ie.value.set_dpi(
                    DPI::try_from(value.to_u8().ok_or(IEConversionError)?)
                        .map_err(|_| IEConversionError)?,
                );
            }
            Self::TI11(ie) => ie.value = value.to_f32().ok_or(IEConversionError)? as i16,
            Self::TI13(ie) => ie.value = value.to_f32().ok_or(IEConversionError)?,
            Self::TI45(ie) => ie.value.set_scs(!value.is_zero()),
            Self::TI50(ie) => ie.value = value.to_f32().ok_or(IEConversionError)?,
            Self::TI112(ie) => ie.value = value.to_f32().ok_or(IEConversionError)?,
            Self::TI136(ie) => ie.value = value.to_f32().ok_or(IEConversionError)? as _,
            Self::TI137(ie) => ie.value = value.to_f32().ok_or(IEConversionError)? as _,
            Self::TI138(ie) => ie.value = value.to_f32().ok_or(IEConversionError)? as _,
            Self::TI139(ie) => ie.value = value.to_f32().ok_or(IEConversionError)? as _,
            Self::TI200(ie) => ie.value = value.to_f32().ok_or(IEConversionError)? as _,
            Self::TI201(ie) => ie.value = value.to_f32().ok_or(IEConversionError)? as _,
            Self::TI202(ie) => ie.value = value.to_f32().ok_or(IEConversionError)? as _,
            Self::TI203(ie) => ie.value = value.to_f32().ok_or(IEConversionError)? as _,
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
            SmallIE::TI11(ie) => ie.value as _,
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
            SmallIE::TI136(ie) => ie.value as _,
            SmallIE::TI137(ie) => ie.value as _,
            SmallIE::TI138(ie) => ie.value as _,
            SmallIE::TI139(ie) => ie.value as _,
            SmallIE::TI200(ie) => ie.value as _,
            SmallIE::TI201(ie) => ie.value as _,
            SmallIE::TI202(ie) => ie.value as _,
            SmallIE::TI203(ie) => ie.value as _,
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
            (Self::TI11(ie), Self::TI11(ie_src)) => *ie = ie_src,
            (Self::TI13(ie), Self::TI13(ie_src)) => *ie = ie_src,
            (Self::TI45(ie), Self::TI45(ie_src)) => *ie = ie_src,
            (Self::TI50(ie), Self::TI50(ie_src)) => *ie = ie_src,
            (Self::TI112(ie), Self::TI112(ie_src)) => *ie = ie_src,
            (Self::TI136(ie), Self::TI136(ie_src)) => *ie = ie_src,
            (Self::TI137(ie), Self::TI137(ie_src)) => *ie = ie_src,
            (Self::TI138(ie), Self::TI138(ie_src)) => *ie = ie_src,
            (Self::TI139(ie), Self::TI139(ie_src)) => *ie = ie_src,
            (Self::TI200(ie), Self::TI200(ie_src)) => *ie = ie_src,
            (Self::TI201(ie), Self::TI201(ie_src)) => *ie = ie_src,
            (Self::TI202(ie), Self::TI202(ie_src)) => *ie = ie_src,
            (Self::TI203(ie), Self::TI203(ie_src)) => *ie = ie_src,
            _ => return Err(IEConversionError),
        }

        Ok(())
    }
}

impl TryUpdateFrom<SmallIE> for TI1 {
    type Error = IEConversionError;

    fn try_update_from(&mut self, value: SmallIE) -> Result<(), Self::Error> {
        match value {
            SmallIE::TI1(ie) => *self = ie,
            SmallIE::TI3(ie) => {
                *self.value.mut_qds_raw() = ie.value.qds_raw();
                self.value.set_spi(ie.value.dpi() == DPI::On);
            }
            _ => return Err(IEConversionError),
        }
        Ok(())
    }
}

impl TryUpdateFrom<SmallIE> for TI3 {
    type Error = IEConversionError;

    fn try_update_from(&mut self, value: SmallIE) -> Result<(), Self::Error> {
        match value {
            SmallIE::TI3(ie) => *self = ie,
            SmallIE::TI1(ie_src) => {
                *self.value.mut_qds_raw() = ie_src.value.qds_raw();
                self.value.set_dpi(ie_src.value.spi().into());
            }
            _ => return Err(IEConversionError),
        }
        Ok(())
    }
}

macro_rules! try_update_from_smie {
    ($($typ:ident,)*) => {
        $(
            impl TryUpdateFrom<SmallIE> for $typ {
                type Error = IEConversionError;

                fn try_update_from(&mut self, value: SmallIE) -> Result<(), Self::Error> {
                    match value {
                        SmallIE::$typ(ie) => *self = ie,
                        _ => return Err(IEConversionError),
                    }
                    Ok(())
                }
            }
        )*
    }
}

try_update_from_smie!(
    TI11, TI13, TI45, TI50, TI112, TI136, TI137, TI138, TI139, TI200, TI201, TI202, TI203,
);

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

impl TryFrom<SmallIE> for u32 {

    type Error = IEConversionError;
    #[allow(clippy::useless_conversion)]
    fn try_from(value: SmallIE) -> Result<Self, Self::Error> {
        match value {
            SmallIE::TI1(v) => Ok(v.value.spi().into()),
            SmallIE::TI3(v) => Ok(v.value.dpi() as Self),
            SmallIE::TI45(v) => Ok(v.value.scs().into()),
            SmallIE::TI11(v) => v.value.try_into().map_err(|_| IEConversionError),
            SmallIE::TI136(v) => v.value.try_into().map_err(|_| IEConversionError),
            SmallIE::TI137(v) => v.value.try_into().map_err(|_| IEConversionError),
            SmallIE::TI138(v) => v.value.try_into().map_err(|_| IEConversionError),
            SmallIE::TI139(v) => v.value.try_into().map_err(|_| IEConversionError),
            SmallIE::TI200(v) => v.value.try_into().map_err(|_| IEConversionError),
            SmallIE::TI201(v) => v.value.try_into().map_err(|_| IEConversionError),
            SmallIE::TI202(v) => v.value.try_into().map_err(|_| IEConversionError),
            SmallIE::TI203(v) => v.value.try_into().map_err(|_| IEConversionError),
            SmallIE::TI13(_) | SmallIE::TI50(_) | SmallIE::TI112(_) => Err(IEConversionError),
        }
    }
}

impl From<M_DP_NA_1> for u32 {
    fn from(value: M_DP_NA_1) -> Self {
        value.value.dpi() as Self
    }
}

impl From<bool> for C_SC_NA_1 {
    fn from(value: bool) -> Self {
        let mut res = Self::default();
        res.value.set_scs(value);
        res
    }
}

impl From<bool> for M_SP_NA_1 {
    fn from(value: bool) -> Self {
        Self {
            value: *SIQ::default().set_spi(value),
        }
    }
}

impl TryFrom<u8> for M_DP_NA_1 {
    type Error = IEConversionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Self {
            value: *DIQ::default().set_dpi(DPI::try_from(value).map_err(|_| IEConversionError)?),
        })
    }
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

impl TryFrom<f32> for M_SP_NA_1 {
    type Error = IEConversionError;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.fract() != 0.0 || !(0.0..=1.0).contains(&value) {
            return Err(IEConversionError);
        }
        Ok(Self::from(value != 0.0))
    }
}

impl TryFrom<f32> for M_DP_NA_1 {
    type Error = IEConversionError;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.fract() != 0.0 || !(0.0..=255.0).contains(&value) {
            return Err(IEConversionError);
        }
        Self::try_from(value as u8)
    }
}

impl TryFrom<f32> for C_SC_NA_1 {
    type Error = IEConversionError;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.fract() != 0.0 || !(0.0..=1.0).contains(&value) {
            return Err(IEConversionError);
        }
        Ok(Self::from(value != 0.0))
    }
}

macro_rules! impl_from_for_ie {
    ($($typ:ident,)*) => {
        $(
            impl From<f32> for $typ {
                fn from(value: f32) -> Self {
                    Self {
                        value: value as _,
                        ..Default::default()
                    }
                }
            }
        )*
    }
}

impl_from_for_ie!(
    M_ME_NE_1, C_SE_NC_1, M_ME_NB_1, P_ME_NC_1, TI136, TI137, TI138, TI139, TI200, TI201, TI202,
    TI203,
);
