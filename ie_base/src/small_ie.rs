use const_default::ConstDefault;

use crate::{
    generic_ie::IEMeta, QualityDescriptor, C_SC_NA_1, C_SE_NB_1, C_SE_NC_1, M_DP_NA_1, M_ME_NB_1,
    M_ME_NE_1, M_SP_NA_1, P_ME_NC_1, TI136, TI137, TI138, TI139, TI200, TI201, TI202, TI203,
};

pub type TI1 = M_SP_NA_1;
pub type TI3 = M_DP_NA_1;
pub type TI11 = M_ME_NB_1;
pub type TI13 = M_ME_NE_1;
pub type TI45 = C_SC_NA_1;
pub type TI49 = C_SE_NB_1;
pub type TI50 = C_SE_NC_1;
pub type TI112 = P_ME_NC_1;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum SmallIE {
    TI1(M_SP_NA_1),
    TI3(M_DP_NA_1),
    TI11(M_ME_NB_1),
    TI13(M_ME_NE_1),
    TI45(C_SC_NA_1),
    TI49(C_SE_NB_1),
    TI50(C_SE_NC_1),
    TI112(P_ME_NC_1),
    TI136(TI136),
    TI137(TI137),
    TI138(TI138),
    TI139(TI139),
    TI200(TI200),
    TI201(TI201),
    TI202(TI202),
    TI203(TI203),
}

enum IEType {
    TI1 = 1,
    TI3 = 3,
    TI11 = 11,
    TI13 = 13,
    TI45 = 45,
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

const fn ie_type(typecode: u8) -> Result<IEType, ()> {
    match typecode {
        1 => Ok(IEType::TI1),
        3 => Ok(IEType::TI3),
        11 => Ok(IEType::TI11),
        13 => Ok(IEType::TI13),
        45 => Ok(IEType::TI45),
        49 => Ok(IEType::TI49),
        50 => Ok(IEType::TI50),
        112 => Ok(IEType::TI112),
        136 => Ok(IEType::TI136),
        137 => Ok(IEType::TI137),
        138 => Ok(IEType::TI138),
        139 => Ok(IEType::TI139),
        200 => Ok(IEType::TI200),
        201 => Ok(IEType::TI201),
        202 => Ok(IEType::TI202),
        203 => Ok(IEType::TI203),
        _ => Err(()),
    }
}

impl TryFrom<u8> for IEType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
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
                type Error = ();
                fn try_from(value: SmallIE) -> Result<Self, ()> {
                    match value {
                        SmallIE::$small(it) => Ok(it),
                        _ => Err(()),
                    }
                }
            }
        )*
    }
}

converts!(
    M_SP_NA_1 <=> TI1,
    M_DP_NA_1 <=> TI3,
    M_ME_NB_1 <=> TI11,
    M_ME_NE_1 <=> TI13,
    C_SC_NA_1 <=> TI45,
    C_SE_NB_1 <=> TI49,
    C_SE_NC_1 <=> TI50,
    P_ME_NC_1 <=> TI112,
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
    pub const fn default_for_typecode(typecode: u8) -> Option<Self> {
        match ie_type(typecode) {
            Ok(IEType::TI1) => Some(Self::TI1(ConstDefault::DEFAULT)),
            Ok(IEType::TI3) => Some(Self::TI3(ConstDefault::DEFAULT)),
            Ok(IEType::TI11) => Some(Self::TI11(ConstDefault::DEFAULT)),
            Ok(IEType::TI13) => Some(Self::TI13(ConstDefault::DEFAULT)),
            Ok(IEType::TI45) => Some(Self::TI45(ConstDefault::DEFAULT)),
            Ok(IEType::TI49) => Some(Self::TI49(ConstDefault::DEFAULT)),
            Ok(IEType::TI50) => Some(Self::TI50(ConstDefault::DEFAULT)),
            Ok(IEType::TI112) => Some(Self::TI112(ConstDefault::DEFAULT)),
            Ok(IEType::TI136) => Some(Self::TI136(ConstDefault::DEFAULT)),
            Ok(IEType::TI137) => Some(Self::TI137(ConstDefault::DEFAULT)),
            Ok(IEType::TI138) => Some(Self::TI138(ConstDefault::DEFAULT)),
            Ok(IEType::TI139) => Some(Self::TI139(ConstDefault::DEFAULT)),
            Ok(IEType::TI200) => Some(Self::TI200(ConstDefault::DEFAULT)),
            Ok(IEType::TI201) => Some(Self::TI201(ConstDefault::DEFAULT)),
            Ok(IEType::TI202) => Some(Self::TI202(ConstDefault::DEFAULT)),
            Ok(IEType::TI203) => Some(Self::TI203(ConstDefault::DEFAULT)),
            Err(()) => None,
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
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        map_small_ie!(self, T, |&mut it| -> (&mut [u8]) it.as_bytes_mut())
    }

    /// Returns bytes to underlying value
    /// Not includes typecode
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        map_small_ie!(self, T, |&it| -> (&[u8]) it.as_bytes())
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
            | Self::TI49(_)
            | Self::TI50(_)
            | Self::TI112(_) => None,
        }
    }

    #[must_use]
    pub fn try_change_typecode(self, new_typecode: u8) -> Option<Self> {
        match (self, new_typecode) {
            (Self::TI1(ie), 1) => Some(Self::TI1(ie)),
            _ => todo!(),
        }
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
            let mut ie_buf: IEBuf = ie.into();
            let ref_ie: &SmallIE = (&ie_buf).try_into().unwrap();
            assert_eq!(ie, *ref_ie);
            let mut_ref_ie: &mut SmallIE = (&mut ie_buf).try_into().unwrap();
            assert_eq!(ie, *mut_ref_ie);
            let owned_ie: SmallIE = ie_buf.try_into().unwrap();
            assert_eq!(ie, owned_ie);
        }
    }

    fn test_bad_iebuf_by_typecode(code: u8) {
        let mut ie_buf = IEBuf([1; core::mem::size_of::<IEBuf>()]);
        ie_buf.0[0] = code;
        let ref_ie: Result<&SmallIE, _> = (&ie_buf).try_into();
        if let Ok(ref_ie) = ref_ie {
            core::hint::black_box(*ref_ie); // miri
        }
        assert!(ref_ie.is_err());
        let mut_ref_ie: Result<&mut SmallIE, _> = (&mut ie_buf).try_into();
        if let Ok(mut_ref_ie) = ref_ie {
            core::hint::black_box(*mut_ref_ie); // miri
        }
        assert!(mut_ref_ie.is_err());
        let owned_ie: Result<SmallIE, _> = ie_buf.try_into();
        assert!(owned_ie.is_err());
    }

    #[test]
    fn test_bad_iebuf() {
        let typecodes: Vec<_> = SmallIE::iter().map(|ie| ie.typecode()).collect();
        for code in 0..=255 {
            if typecodes.iter().all(|&tc| tc != code) {
                test_bad_iebuf_by_typecode(code);
            }
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
        assert_eq!(
            Some(&SmallIE::TI3(Default::default())),
            (&ie_buf).try_into().ok()
        );
    }
}
