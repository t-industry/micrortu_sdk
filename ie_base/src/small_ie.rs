use const_default::ConstDefault;
use zerocopy::AsBytes;

use crate::{QualityDescriptor, C_SC_NA_1, C_SE_NC_1, M_DP_NA_1, M_ME_NE_1, M_SP_NA_1, P_ME_NC_1};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum SmallIE {
    TI1(M_SP_NA_1) = 1,
    TI3(M_DP_NA_1) = 3,
    TI13(M_ME_NE_1) = 13,
    TI45(C_SC_NA_1) = 45,
    TI50(C_SE_NC_1) = 50,
    TI112(P_ME_NC_1) = 112,
}

impl Default for SmallIE {
    fn default() -> Self {
        Self::TI1(ConstDefault::DEFAULT)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BufferTooSmall;

impl SmallIE {
    #[must_use]
    pub const fn typecode(&self) -> u8 {
        match self {
            Self::TI1(_) => 1,
            Self::TI3(_) => 3,
            Self::TI13(_) => 13,
            Self::TI45(_) => 45,
            Self::TI50(_) => 50,
            Self::TI112(_) => 112,
        }
    }

    #[must_use]
    pub const fn ti1(value: M_SP_NA_1) -> Self {
        Self::TI1(value)
    }
    #[must_use]
    pub const fn ti3(value: M_DP_NA_1) -> Self {
        Self::TI3(value)
    }
    #[must_use]
    pub const fn ti13(value: M_ME_NE_1) -> Self {
        Self::TI13(value)
    }
    #[must_use]
    pub const fn ti45(value: C_SC_NA_1) -> Self {
        Self::TI45(value)
    }
    #[must_use]
    pub const fn ti50(value: C_SE_NC_1) -> Self {
        Self::TI50(value)
    }
    #[must_use]
    pub const fn ti112(value: P_ME_NC_1) -> Self {
        Self::TI112(value)
    }

    #[must_use]
    pub const fn default_for_typecode(typecode: u8) -> Option<Self> {
        match typecode {
            1 => Some(Self::TI1(ConstDefault::DEFAULT)),
            3 => Some(Self::TI3(ConstDefault::DEFAULT)),
            13 => Some(Self::TI13(ConstDefault::DEFAULT)),
            45 => Some(Self::TI45(ConstDefault::DEFAULT)),
            50 => Some(Self::TI50(ConstDefault::DEFAULT)),
            112 => Some(Self::TI112(ConstDefault::DEFAULT)),
            _ => None,
        }
    }

    /// Returns bytes to underlying value
    /// Not includes typecode
    #[must_use]
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        match self {
            Self::TI1(it) => it.as_bytes_mut(),
            Self::TI3(it) => it.as_bytes_mut(),
            Self::TI13(it) => it.as_bytes_mut(),
            Self::TI45(it) => it.as_bytes_mut(),
            Self::TI50(it) => it.as_bytes_mut(),
            Self::TI112(it) => it.as_bytes_mut(),
        }
    }

    /// Returns bytes to underlying value
    /// Not includes typecode
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::TI1(it) => it.as_bytes(),
            Self::TI3(it) => it.as_bytes(),
            Self::TI13(it) => it.as_bytes(),
            Self::TI45(it) => it.as_bytes(),
            Self::TI50(it) => it.as_bytes(),
            Self::TI112(it) => it.as_bytes(),
        }
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
            Self::TI13(ie) => Some(&ie.qds),
            Self::TI45(_) | Self::TI50(_) | Self::TI112(_) => None,
        }
    }

    #[must_use]
    pub fn try_get_qds_mut(&mut self) -> Option<&mut dyn QualityDescriptor> {
        match self {
            Self::TI1(ie) => Some(&mut ie.value),
            Self::TI3(ie) => Some(&mut ie.value),
            Self::TI13(ie) => Some(&mut ie.qds),
            Self::TI45(_) | Self::TI50(_) | Self::TI112(_) => None,
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
