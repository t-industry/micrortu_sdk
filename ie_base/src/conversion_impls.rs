#[allow(unused_imports)]
use num::{Float as _, Num, ToPrimitive};

use crate::{measurement::DPI, qds::QualityDescriptorHolder, *};

enum ValueBridge {
    F32(f32),
    I64(i64),
    U64(u64),
}

impl<T: Num + ToPrimitive> TryUpdateFrom<T> for SmallIE {
    type Error = IEConversionError;

    fn try_update_from(&mut self, value: T) -> Result<(), Self::Error> {
        let bridge_val = if let Some(v) = value.to_i64() {
            ValueBridge::I64(v)
        } else if let Some(v) = value.to_u64() {
            ValueBridge::U64(v)
        } else if let Some(v) = value.to_f32() {
            ValueBridge::F32(v)
        } else {
            return Err(IEConversionError);
        };

        self.apply_bridge_value(bridge_val);

        Ok(())
    }
}

impl SmallIE {
    /// Updates from `u64` or `i64` or `f32`.
    pub fn try_update_from_str(&mut self, value: &str) -> Result<(), crate::ParseError> {
        let bridge_val = if let Ok(v) = value.parse::<u64>() {
            ValueBridge::U64(v)
        } else if let Ok(v) = value.parse::<i64>() {
            ValueBridge::I64(v)
        } else if let Ok(v) = value.parse::<f32>() {
            ValueBridge::F32(v)
        } else {
            return Err(crate::ParseError);
        };
        self.apply_bridge_value(bridge_val);
        Ok(())
    }

    #[inline]
    pub fn update_from(&mut self, other: &Self) {
        if core::mem::discriminant(self) == core::mem::discriminant(&other) {
            *self = *other;
            return;
        }

        let bridge_val = other.extract_bridge_value();
        let qds = other.extract_qds();
        let qos_raw = other.extract_qos_byte();

        self.apply_bridge_value(bridge_val);

        if let Some(q) = qds {
            self.apply_qds(&q);
        }

        if let Some(byte) = qos_raw {
            self.apply_qos_byte(byte);
        }
    }
}

impl TryUpdateFrom<Self> for SmallIE {
    type Error = core::convert::Infallible;

    fn try_update_from(&mut self, value: Self) -> Result<(), Self::Error> {
        self.update_from(&value);
        Ok(())
    }
}

impl SmallIE {
    #[inline(always)]
    pub fn change_typecode(&mut self, new: u8) -> Result<(), InvalidIeType> {
        IeType::new(new).map(|new| self.change_type(new))
    }
    #[inline(always)]
    pub fn change_type(&mut self, new: IeType) {
        let mut value = Self::default_for_type(new);
        value.update_from(self);
        *self = value;
    }

    #[must_use]
    #[inline(always)]
    pub fn update_element_with<T: crate::generic_ie::IEMeta>(element: T, value: &Self) -> T {
        let mut this: Self = element.into();
        this.update_from(value);
        this.extract_ti().expect("We just set this value")
    }

    fn extract_bridge_value(&self) -> ValueBridge {
        match self {
            Self::TI1(ie) => ValueBridge::U64(ie.value.spi() as u64),
            Self::TI3(ie) => ValueBridge::U64(ie.value.dpi() as u64),
            Self::TI11(ie) => ValueBridge::I64(ie.value as i64),
            Self::TI13(ie) => ValueBridge::F32(ie.value),
            Self::TI45(ie) => ValueBridge::U64(ie.value.scs() as u64),
            Self::TI46(ie) => ValueBridge::U64(ie.dco.dcs() as u64),
            Self::TI49(ie) => ValueBridge::I64(ie.value as i64),
            Self::TI50(ie) => ValueBridge::F32(ie.value),
            Self::TI112(ie) => ValueBridge::F32(ie.value),
            Self::TI136(ie) => ValueBridge::U64(ie.value as u64),
            Self::TI137(ie) => ValueBridge::I64(ie.value as i64),
            Self::TI138(ie) => ValueBridge::U64(ie.value),
            Self::TI139(ie) => ValueBridge::I64(ie.value),
            Self::TI200(ie) => ValueBridge::U64(ie.value as u64),
            Self::TI201(ie) => ValueBridge::I64(ie.value as i64),
            Self::TI202(ie) => ValueBridge::U64(ie.value),
            Self::TI203(ie) => ValueBridge::I64(ie.value as i64),
        }
    }

    #[inline]
    fn apply_bridge_value(&mut self, bridge: ValueBridge) {
        #[inline(always)]
        fn uclamp<T: Into<i64> + TryFrom<i64>>(v: u64, min: T, max: T) -> T {
            match (v as i64).clamp(min.into(), max.into()).try_into() {
                Ok(v) => v,
                Err(_e) => unreachable!(),
            }
        }
        #[inline(always)]
        fn iclamp<T: Into<i64> + TryFrom<i64>>(v: i64, min: T, max: T) -> T {
            match v.clamp(min.into(), max.into()).try_into() {
                Ok(v) => v,
                Err(_e) => unreachable!(),
            }
        }

        match (self, bridge) {
            (Self::TI1(ie), ValueBridge::U64(v)) => _ = ie.value.set_spi(v != 0),
            (Self::TI1(ie), ValueBridge::I64(v)) => _ = ie.value.set_spi(v != 0),
            (Self::TI1(ie), ValueBridge::F32(v)) => _ = ie.value.set_spi(v != 0.0),
            (Self::TI3(ie), ValueBridge::U64(1)) => _ = ie.value.set_dpi(DPI::On),
            (Self::TI3(ie), ValueBridge::I64(1)) => _ = ie.value.set_dpi(DPI::On),
            (Self::TI3(ie), ValueBridge::F32(1.0)) => _ = ie.value.set_dpi(DPI::On),
            (Self::TI3(ie), ValueBridge::U64(_)) => _ = ie.value.set_dpi(DPI::Off),
            (Self::TI3(ie), ValueBridge::I64(_)) => _ = ie.value.set_dpi(DPI::Off),
            (Self::TI3(ie), ValueBridge::F32(_)) => _ = ie.value.set_dpi(DPI::Off),
            (Self::TI11(ie), ValueBridge::U64(v)) => ie.value = uclamp(v, i16::MIN, i16::MAX),
            (Self::TI11(ie), ValueBridge::I64(v)) => ie.value = iclamp(v, i16::MIN, i16::MAX),
            (Self::TI11(ie), ValueBridge::F32(v)) => ie.value = v as i16,
            (Self::TI13(ie), ValueBridge::U64(v)) => ie.value = v as f32,
            (Self::TI13(ie), ValueBridge::I64(v)) => ie.value = v as f32,
            (Self::TI13(ie), ValueBridge::F32(v)) => ie.value = v,
            (Self::TI45(ie), ValueBridge::U64(v)) => ie.value.set_scs(v != 0),
            (Self::TI45(ie), ValueBridge::I64(v)) => ie.value.set_scs(v != 0),
            (Self::TI45(ie), ValueBridge::F32(v)) => ie.value.set_scs(v != 0.0),
            (Self::TI46(ie), ValueBridge::U64(1)) => _ = ie.dco.set_dcs(DCS::On),
            (Self::TI46(ie), ValueBridge::I64(1)) => _ = ie.dco.set_dcs(DCS::On),
            (Self::TI46(ie), ValueBridge::F32(1.0)) => _ = ie.dco.set_dcs(DCS::On),
            (Self::TI46(ie), ValueBridge::F32(_)) => _ = ie.dco.set_dcs(DCS::Off),
            (Self::TI46(ie), ValueBridge::U64(_)) => _ = ie.dco.set_dcs(DCS::Off),
            (Self::TI46(ie), ValueBridge::I64(_)) => _ = ie.dco.set_dcs(DCS::Off),
            (Self::TI49(ie), ValueBridge::U64(v)) => ie.value = uclamp(v, i16::MIN, i16::MAX),
            (Self::TI49(ie), ValueBridge::I64(v)) => ie.value = iclamp(v, i16::MIN, i16::MAX),
            (Self::TI49(ie), ValueBridge::F32(v)) => ie.value = uclamp(v as u64, i16::MIN, i16::MAX),
            (Self::TI50(ie), ValueBridge::F32(v)) => ie.value = v,
            (Self::TI50(ie), ValueBridge::U64(v)) => ie.value = v as f32,
            (Self::TI50(ie), ValueBridge::I64(v)) => ie.value = v as f32,
            (Self::TI112(ie), ValueBridge::F32(v)) => ie.value = v,
            (Self::TI112(ie), ValueBridge::U64(v)) => ie.value = v as f32,
            (Self::TI112(ie), ValueBridge::I64(v)) => ie.value = v as f32,

            (Self::TI136(ie), ValueBridge::U64(v)) => ie.value = uclamp(v, 0, u32::MAX),
            (Self::TI137(ie), ValueBridge::U64(v)) => ie.value = uclamp(v, i32::MIN, i32::MAX),
            (Self::TI138(ie), ValueBridge::U64(v)) => ie.value = v,
            (Self::TI139(ie), ValueBridge::U64(v)) => ie.value = v.clamp(0, i64::MAX as u64) as i64,
            (Self::TI200(ie), ValueBridge::U64(v)) => ie.value = uclamp(v, 0, u32::MAX),
            (Self::TI201(ie), ValueBridge::U64(v)) => ie.value = uclamp(v, i32::MIN, i32::MAX),
            (Self::TI202(ie), ValueBridge::U64(v)) => ie.value = v,
            (Self::TI203(ie), ValueBridge::U64(v)) => ie.value = v.clamp(0, i64::MAX as u64) as i64,
            (Self::TI136(ie), ValueBridge::I64(v)) => ie.value = iclamp(v, 0, u32::MAX),
            (Self::TI137(ie), ValueBridge::I64(v)) => ie.value = iclamp(v, i32::MIN, i32::MAX),
            (Self::TI138(ie), ValueBridge::I64(v)) => ie.value = v.clamp(0, i64::MAX) as u64,
            (Self::TI139(ie), ValueBridge::I64(v)) => ie.value = v,
            (Self::TI200(ie), ValueBridge::I64(v)) => ie.value = iclamp(v, 0, u32::MAX) as u32,
            (Self::TI201(ie), ValueBridge::I64(v)) => ie.value = iclamp(v, i32::MIN, i32::MAX),
            (Self::TI202(ie), ValueBridge::I64(v)) => ie.value = v.clamp(0, i64::MAX) as u64,
            (Self::TI203(ie), ValueBridge::I64(v)) => ie.value = v,

            (Self::TI136(ie), ValueBridge::F32(v)) => ie.value = v as _,
            (Self::TI137(ie), ValueBridge::F32(v)) => ie.value = v as _,
            (Self::TI138(ie), ValueBridge::F32(v)) => ie.value = v as _,
            (Self::TI139(ie), ValueBridge::F32(v)) => ie.value = v as _,
            (Self::TI200(ie), ValueBridge::F32(v)) => ie.value = v as _,
            (Self::TI201(ie), ValueBridge::F32(v)) => ie.value = v as _,
            (Self::TI202(ie), ValueBridge::F32(v)) => ie.value = v as _,
            (Self::TI203(ie), ValueBridge::F32(v)) => ie.value = v as _,
        }
    }

    fn extract_qos_byte(&self) -> Option<u8> {
        match self {
            Self::TI49(ie) => Some(ie.qos.0),
            Self::TI50(ie) => Some(ie.qos.0),
            Self::TI200(ie) => Some(ie.qos.0),
            Self::TI201(ie) => Some(ie.qos.0),
            Self::TI202(ie) => Some(ie.qos.0),
            Self::TI203(ie) => Some(ie.qos.0),
            _ => None,
        }
    }

    fn extract_qds(&self) -> Option<crate::RawQualityDescriptor> {
        match self {
            Self::TI1(ie) => Some(ie.value.qds_raw()),
            Self::TI3(ie) => Some(ie.value.qds_raw()),
            Self::TI11(ie) => Some(ie.qds.raw),
            Self::TI13(ie) => Some(ie.qds.raw),
            Self::TI136(ie) => Some(ie.qds.raw),
            Self::TI137(ie) => Some(ie.qds.raw),
            Self::TI138(ie) => Some(ie.qds.raw),
            Self::TI139(ie) => Some(ie.qds.raw),
            Self::TI45(_)
            | Self::TI46(_)
            | Self::TI49(_)
            | Self::TI50(_)
            | Self::TI112(_)
            | Self::TI200(_)
            | Self::TI201(_)
            | Self::TI202(_)
            | Self::TI203(_) => None,
        }
    }

    fn apply_qds(&mut self, qds: &dyn QualityDescriptor) {
        match self {
            Self::TI1(ie) => ie.value.update_from(qds),
            Self::TI3(ie) => ie.value.update_from(qds),
            Self::TI11(ie) => ie.qds.update_from(qds),
            Self::TI13(ie) => ie.qds.update_from(qds),
            Self::TI136(ie) => ie.qds.update_from(qds),
            Self::TI137(ie) => ie.qds.update_from(qds),
            Self::TI138(ie) => ie.qds.update_from(qds),
            Self::TI139(ie) => ie.qds.update_from(qds),
            Self::TI45(_)
            | Self::TI46(_)
            | Self::TI49(_)
            | Self::TI50(_)
            | Self::TI112(_)
            | Self::TI200(_)
            | Self::TI201(_)
            | Self::TI202(_)
            | Self::TI203(_) => (),
        }
    }

    fn apply_qos_byte(&mut self, raw: u8) {
        match self {
            Self::TI49(ie) => ie.qos.0 = raw,
            Self::TI50(ie) => ie.qos.0 = raw,
            Self::TI200(ie) => ie.qos.0 = raw,
            Self::TI201(ie) => ie.qos.0 = raw,
            Self::TI202(ie) => ie.qos.0 = raw,
            Self::TI203(ie) => ie.qos.0 = raw,
            Self::TI1(_)
            | Self::TI3(_)
            | Self::TI11(_)
            | Self::TI13(_)
            | Self::TI45(_)
            | Self::TI46(_)
            | Self::TI112(_)
            | Self::TI136(_)
            | Self::TI137(_)
            | Self::TI138(_)
            | Self::TI139(_) => (),
        }
    }
}

macro_rules! try_update_from_smie {
    ($($typ:ident,)*) => {
        $(
            impl TryUpdateFrom<SmallIE> for $typ {
                type Error = ::core::convert::Infallible;

                fn try_update_from(&mut self, value: SmallIE) -> Result<(), Self::Error> {
                    *self = SmallIE::update_element_with(*self, &value);
                    Ok(())
                }
            }
        )*
    }
}

try_update_from_smie!(
    TI1, TI3, TI11, TI13, TI45, TI46, TI49, TI50, TI112, TI136, TI137, TI138, TI139, TI200, TI201,
    TI202, TI203,
);

mod impl_bool {
    use super::*;

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
    impl From<bool> for DPI {
        fn from(value: bool) -> Self {
            if value {
                Self::On
            } else {
                Self::Off
            }
        }
    }
}

mod impl_u32 {
    use super::*;

    impl TryFrom<SmallIE> for u32 {
        type Error = IEConversionError;
        #[allow(clippy::useless_conversion)]
        fn try_from(value: SmallIE) -> Result<Self, Self::Error> {
            match value {
                SmallIE::TI1(v) => Ok(v.value.spi().into()),
                SmallIE::TI3(v) => Ok(v.value.dpi() as Self),
                SmallIE::TI45(v) => Ok(v.value.scs().into()),
                SmallIE::TI46(v) => Ok(v.dco.dcs() as Self),
                SmallIE::TI49(v) => v.value.try_into().map_err(|_| IEConversionError),
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
}

mod impl_u8 {
    use crate::DCO;

    use super::*;

    impl TryFrom<u8> for TI3 {
        type Error = IEConversionError;

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            Ok(Self {
                value: *DIQ::default()
                    .set_dpi(DPI::try_from(value).map_err(|_| IEConversionError)?),
            })
        }
    }

    impl TryFrom<u8> for TI46 {
        type Error = IEConversionError;

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            Ok(Self {
                dco: *DCO::default().set_dcs(DCS::try_from(value).map_err(|_| IEConversionError)?),
            })
        }
    }
}

mod impl_f32 {
    use super::*;

    impl From<SmallIE> for f32 {
        fn from(value: SmallIE) -> Self {
            match value {
                SmallIE::TI1(ie) if ie.value.spi() => 1.0,
                SmallIE::TI1(_) => 0.0,
                SmallIE::TI3(ie) => (ie.value.dpi() as u8).to_f32().unwrap_or_default(),
                SmallIE::TI11(ie) => ie.value as _,
                SmallIE::TI13(ie) => ie.value,
                SmallIE::TI45(ie) if ie.value.scs() => 1.0,
                SmallIE::TI45(_) => 0.0,
                SmallIE::TI46(ie) => (ie.dco.dcs() as u8).to_f32().unwrap_or_default(),
                SmallIE::TI49(ie) => ie.value as _,
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

    impl TryFrom<f32> for TI46 {
        type Error = IEConversionError;
        fn try_from(value: f32) -> Result<Self, Self::Error> {
            if value.fract() != 0.0 || !(0.0..=255.0).contains(&value) {
                return Err(IEConversionError);
            }
            Self::try_from(value as u8)
        }
    }

    macro_rules! impl_from_f32_for_ie {
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

    impl_from_f32_for_ie!(
        M_ME_NE_1, C_SE_NB_1, C_SE_NC_1, M_ME_NB_1, P_ME_NC_1, TI136, TI137, TI138, TI139, TI200,
        TI201, TI202, TI203,
    );
}
