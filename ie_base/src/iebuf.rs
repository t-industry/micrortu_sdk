use zerocopy::{FromBytes, FromZeros, IntoBytes};

use crate::{IeType, SmallIE};
use core::mem::size_of;

static_assertions::assert_eq_size!(SmallIE, IEBuf);
static_assertions::assert_eq_align!(SmallIE, IEBuf);

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, PartialEq, IntoBytes, FromBytes)]
pub struct IEBuf(pub [u8; size_of::<SmallIE>()]);

impl IEBuf {
    #[must_use]
    pub fn is_valid(self) -> bool {
        IeType::new(self.0[0]).is_ok()
    }

    #[must_use]
    pub fn terminator() -> Self {
        Self([0; size_of::<SmallIE>()])
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct IEDeserializationError;

impl From<crate::InvalidIeType> for IEDeserializationError {
    fn from(_: crate::InvalidIeType) -> Self {
        Self
    }
}

impl TryFrom<IEBuf> for SmallIE {
    type Error = IEDeserializationError;
    fn try_from(value: IEBuf) -> Result<Self, Self::Error> {
        let ie_type = IeType::new(value.0[0])?;
        let mut new = Self::default_for_type(ie_type);
        let bytes = &value.0[1..];
        let target = new.as_mut_bytes();
        target.copy_from_slice(&bytes[..target.len()]);

        Ok(new)
    }
}

impl From<SmallIE> for IEBuf {
    #[inline]
    fn from(value: SmallIE) -> Self {
        let d = value.typecode();
        let mut this = Self::new_zeroed();
        let bytes = Self::as_mut_bytes(&mut this);
        bytes[0] = d;
        value
            .copy_to_slice(&mut bytes[1..])
            .expect("Space should be sufficient");
        this
    }
}
