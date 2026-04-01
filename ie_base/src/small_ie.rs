use const_default::ConstDefault;

use crate::{
    generic_ie::IEMeta, QualityDescriptor, TI136, TI137, TI138, TI139, TI200, TI201, TI202, TI203,
};

pub type TI1 = crate::M_SP_NA_1;
pub type TI3 = crate::M_DP_NA_1;
pub type TI11 = crate::M_ME_NB_1;
pub type TI13 = crate::M_ME_NE_1;
pub type TI45 = crate::C_SC_NA_1;
pub type TI46 = crate::C_DC_NA_1;
pub type TI49 = crate::C_SE_NB_1;
pub type TI50 = crate::C_SE_NC_1;
pub type TI112 = crate::P_ME_NC_1;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum SmallIE {
    TI1(TI1) = 1,
    TI3(TI3) = 3,
    TI11(TI11) = 11,
    TI13(TI13) = 13,
    TI45(TI45) = 45,
    TI46(TI46) = 46,
    TI49(TI49) = 49,
    TI50(TI50) = 50,
    TI112(TI112) = 112,
    TI136(TI136) = 136,
    TI137(TI137) = 137,
    TI138(TI138) = 138,
    TI139(TI139) = 139,
    TI200(TI200) = 200,
    TI201(TI201) = 201,
    TI202(TI202) = 202,
    TI203(TI203) = 203,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum IeType {
    TI1 = 1,
    TI3 = 3,
    TI11 = 11,
    TI13 = 13,
    TI45 = 45,
    TI46 = 46,
    TI49 = 49,
    TI50 = 50,
    TI112 = 112,
    TI136 = 136,
    TI137 = 137,
    TI138 = 138,
    TI139 = 139,
    TI200 = 200,
    TI201 = 201,
    TI202 = 202,
    TI203 = 203,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BufferTooSmall;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidIeType;

const fn ie_type(typecode: u8) -> Result<IeType, InvalidIeType> {
    IeType::new(typecode)
}

impl IeType {
    #[inline(always)]
    pub const fn new(typecode: u8) -> Result<IeType, InvalidIeType> {
        match typecode {
            1 => Ok(IeType::TI1),
            3 => Ok(IeType::TI3),
            11 => Ok(IeType::TI11),
            13 => Ok(IeType::TI13),
            45 => Ok(IeType::TI45),
            46 => Ok(IeType::TI46),
            49 => Ok(IeType::TI49),
            50 => Ok(IeType::TI50),
            112 => Ok(IeType::TI112),
            136 => Ok(IeType::TI136),
            137 => Ok(IeType::TI137),
            138 => Ok(IeType::TI138),
            139 => Ok(IeType::TI139),
            200 => Ok(IeType::TI200),
            201 => Ok(IeType::TI201),
            202 => Ok(IeType::TI202),
            203 => Ok(IeType::TI203),
            _ => Err(InvalidIeType),
        }
    }
}

impl TryFrom<u8> for IeType {
    type Error = InvalidIeType;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        ie_type(value)
    }
}

impl core::fmt::Display for IeType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}
impl core::str::FromStr for IeType {
    type Err = InvalidIeType;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u8>().map_err(|_| InvalidIeType)?;
        ie_type(value)
    }
}

macro_rules! converts {
    ($($ie:ident <=> $small:ident,)*) => {
        $(
            impl From<$ie> for SmallIE {
                fn from(value: $ie) -> Self {
                    Self::$small(value)
                }
            }

            impl TryFrom<SmallIE> for $ie {
                type Error = InvalidIeType;
                fn try_from(value: SmallIE) -> Result<Self, InvalidIeType> {
                    match value {
                        SmallIE::$small(it) => Ok(it),
                        _ => Err(InvalidIeType),
                    }
                }
            }
        )*
    }
}

converts!(
    TI1 <=> TI1,
    TI3 <=> TI3,
    TI11 <=> TI11,
    TI13 <=> TI13,
    TI45 <=> TI45,
    TI46 <=> TI46,
    TI49 <=> TI49,
    TI50 <=> TI50,
    TI112 <=> TI112,
    TI136 <=> TI136,
    TI137 <=> TI137,
    TI138 <=> TI138,
    TI139 <=> TI139,
    TI200 <=> TI200,
    TI201 <=> TI201,
    TI202 <=> TI202,
    TI203 <=> TI203,
);

impl Default for SmallIE {
    fn default() -> Self {
        Self::TI1(ConstDefault::DEFAULT)
    }
}

macro_rules! map_small_ie {
    ($ie:expr, $f:expr) => {
        match $ie {
            Self::TI1(v) => $f(v),
            Self::TI3(v) => $f(v),
            Self::TI11(v) => $f(v),
            Self::TI13(v) => $f(v),
            Self::TI45(v) => $f(v),
            Self::TI46(v) => $f(v),
            Self::TI49(v) => $f(v),
            Self::TI50(v) => $f(v),
            Self::TI112(v) => $f(v),
            Self::TI136(v) => $f(v),
            Self::TI137(v) => $f(v),
            Self::TI138(v) => $f(v),
            Self::TI139(v) => $f(v),
            Self::TI200(v) => $f(v),
            Self::TI201(v) => $f(v),
            Self::TI202(v) => $f(v),
            Self::TI203(v) => $f(v),
        }
    };
    ($ie:expr, $T:ident, || -> ($r:ty) $body:expr) => {{
        fn f<$T: GetIEType>(_: &$T) -> $r {
            $body
        }
        map_small_ie!($ie, f)
    }};
    ($ie:expr, $T:ident, const || -> ($r:ty) $body:expr) => {{
        const fn f<$T: IEMeta>(_: &$T) -> $r {
            $body
        }
        map_small_ie!($ie, f)
    }};
    ($ie:expr, $T:ident, |&$v:ident| -> ($r:ty) $body:expr) => {{
        fn f<$T: IEMeta>($v: &$T) -> $r {
            $body
        }
        map_small_ie!($ie, f)
    }};
    ($ie:expr, $T:ident, |&mut $v:ident| -> ($r:ty) $body:expr) => {{
        fn f<$T: IEMeta>($v: &mut $T) -> $r {
            $body
        }
        map_small_ie!($ie, f)
    }};
}

impl SmallIE {
    #[must_use]
    pub const fn typecode(&self) -> u8 {
        map_small_ie!(self, T, const || -> (u8) T::TYPECODE)
    }

    #[must_use]
    pub const fn ie_type(&self) -> IeType {
        match map_small_ie!(self, T, const || -> (Result<IeType, InvalidIeType>) IeType::new(T::TYPECODE))
        {
            Ok(v) => v,
            Err(_) => unreachable!(),
        }
    }

    #[must_use]
    pub const fn default_for_type(ie_type: IeType) -> Self {
        match ie_type {
            IeType::TI1 => Self::TI1(ConstDefault::DEFAULT),
            IeType::TI3 => Self::TI3(ConstDefault::DEFAULT),
            IeType::TI11 => Self::TI11(ConstDefault::DEFAULT),
            IeType::TI13 => Self::TI13(ConstDefault::DEFAULT),
            IeType::TI45 => Self::TI45(ConstDefault::DEFAULT),
            IeType::TI46 => Self::TI46(ConstDefault::DEFAULT),
            IeType::TI49 => Self::TI49(ConstDefault::DEFAULT),
            IeType::TI50 => Self::TI50(ConstDefault::DEFAULT),
            IeType::TI112 => Self::TI112(ConstDefault::DEFAULT),
            IeType::TI136 => Self::TI136(ConstDefault::DEFAULT),
            IeType::TI137 => Self::TI137(ConstDefault::DEFAULT),
            IeType::TI138 => Self::TI138(ConstDefault::DEFAULT),
            IeType::TI139 => Self::TI139(ConstDefault::DEFAULT),
            IeType::TI200 => Self::TI200(ConstDefault::DEFAULT),
            IeType::TI201 => Self::TI201(ConstDefault::DEFAULT),
            IeType::TI202 => Self::TI202(ConstDefault::DEFAULT),
            IeType::TI203 => Self::TI203(ConstDefault::DEFAULT),
        }
    }

    #[must_use]
    pub const fn default_for_typecode(typecode: u8) -> Option<Self> {
        match ie_type(typecode) {
            Ok(t) => Some(Self::default_for_type(t)),
            Err(_) => None,
        }
    }

    #[must_use]
    pub const fn size_for_typecode(typecode: u8) -> usize {
        match Self::default_for_typecode(typecode) {
            Some(v) => map_small_ie!(&v, T, const || -> (usize) core::mem::size_of::<T>()),
            None => 0,
        }
    }

    #[must_use]
    pub const fn align_for_typecode(typecode: u8) -> usize {
        match Self::default_for_typecode(typecode) {
            Some(v) => map_small_ie!(&v, T, const || -> (usize) core::mem::align_of::<T>()),
            None => 0,
        }
    }

    /// Returns bytes to underlying value
    /// Not includes typecode
    #[must_use]
    #[inline(always)]
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        map_small_ie!(self, T, |&mut it| -> (&mut [u8]) it.as_mut_bytes())
    }

    /// Returns bytes to underlying value
    /// Not includes typecode
    #[must_use]
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        map_small_ie!(self, T, |&it| -> (&[u8]) it.as_bytes())
    }

    #[must_use]
    #[inline(always)]
    pub fn try_from_typecode_and_bytes(typecode: u8, bytes: &[u8]) -> Option<Self> {
        let bytes = bytes.get(..Self::size_for_typecode(typecode))?;
        let mut value = Self::default_for_typecode(typecode)?;
        value.as_mut_bytes().copy_from_slice(bytes);
        Some(value)
    }

    /// Copies bytes to underlying value
    ///
    /// # Errors
    ///
    /// Returns `SizesNotMatchError` if target buffer size is less then value size
    pub fn copy_to_slice(&self, buf: &mut [u8]) -> Result<(), BufferTooSmall> {
        let bytes = self.as_bytes();
        let target = buf.get_mut(..bytes.len()).ok_or(BufferTooSmall)?;
        target.copy_from_slice(bytes);
        Ok(())
    }

    #[must_use]
    pub fn try_get_qds(&self) -> Option<&dyn QualityDescriptor> {
        match self {
            Self::TI1(ie) => Some(&ie.value),
            Self::TI3(ie) => Some(&ie.value),
            Self::TI11(ie) => Some(&ie.qds),
            Self::TI13(ie) => Some(&ie.qds),
            Self::TI136(ie) => Some(&ie.qds),
            Self::TI137(ie) => Some(&ie.qds),
            Self::TI138(ie) => Some(&ie.qds),
            Self::TI139(ie) => Some(&ie.qds),
            Self::TI200(_)
            | Self::TI201(_)
            | Self::TI202(_)
            | Self::TI203(_)
            | Self::TI45(_)
            | Self::TI46(_)
            | Self::TI49(_)
            | Self::TI50(_)
            | Self::TI112(_) => None,
        }
    }

    #[must_use]
    pub fn try_get_qds_mut(&mut self) -> Option<&mut dyn QualityDescriptor> {
        match self {
            Self::TI1(ie) => Some(&mut ie.value),
            Self::TI3(ie) => Some(&mut ie.value),
            Self::TI11(ie) => Some(&mut ie.qds),
            Self::TI13(ie) => Some(&mut ie.qds),
            Self::TI136(ie) => Some(&mut ie.qds),
            Self::TI137(ie) => Some(&mut ie.qds),
            Self::TI138(ie) => Some(&mut ie.qds),
            Self::TI139(ie) => Some(&mut ie.qds),
            Self::TI200(_)
            | Self::TI201(_)
            | Self::TI202(_)
            | Self::TI203(_)
            | Self::TI45(_)
            | Self::TI46(_)
            | Self::TI49(_)
            | Self::TI50(_)
            | Self::TI112(_) => None,
        }
    }

    #[must_use]
    #[inline(always)]
    pub fn extract_ti<T: IEMeta>(&self) -> Option<T> {
        if T::TYPECODE == self.typecode() {
            let value = T::ref_from_bytes(self.as_bytes());
            Some(*value.expect("Typecodes match"))
        } else {
            None
        }
    }
}

impl core::error::Error for InvalidIeType {}
impl core::fmt::Display for InvalidIeType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Invalid IE type")
    }
}

#[cfg(test)]
mod test {
    use core::ptr::addr_of;
    use std::collections::HashMap;

    use strum::IntoEnumIterator;

    use crate::{IEBuf, SmallIE};

    #[test]
    fn test_default() {
        for ie in SmallIE::iter() {
            let ie_buf: IEBuf = ie.into();
            let owned_ie: SmallIE = ie_buf.try_into().unwrap();
            assert_eq!(ie, owned_ie);
        }
    }

    #[test]
    fn test_typecodes() {
        for ie in SmallIE::iter() {
            // Safety: SmallIE is repr(u8)
            let unsafe_typecode = unsafe { *addr_of!(ie).cast::<u8>() };
            assert_eq!(ie.typecode(), unsafe_typecode);
        }
    }

    #[test]
    fn test_defaults() {
        let defaults: HashMap<u8, SmallIE> =
            SmallIE::iter().map(|ie| (ie.typecode(), ie)).collect();

        for (&code, &ie) in &defaults {
            assert_eq!(Some(ie), SmallIE::default_for_typecode(code));
        }

        for code in 0..=255 {
            if !defaults.contains_key(&code) {
                assert_eq!(None, SmallIE::default_for_typecode(code));
            }
        }
    }

    #[test]
    fn change_type() {
        let ie_t1 = SmallIE::TI1(Default::default());
        let mut ie_buf: IEBuf = ie_t1.into();
        ie_buf.0[0] = 3;
        assert_eq!(
            Some(SmallIE::TI3(Default::default())),
            ie_buf.try_into().ok()
        );
    }
}
