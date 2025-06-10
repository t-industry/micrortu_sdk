use zerocopy::{IntoBytes, FromBytes, FromZeros};

use crate::SmallIE;
use core::mem::size_of;

static_assertions::assert_eq_size!(SmallIE, IEBuf);
static_assertions::assert_eq_align!(SmallIE, IEBuf);

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, PartialEq, IntoBytes, FromBytes)]
pub struct IEBuf(pub [u8; size_of::<SmallIE>()]);

impl IEBuf {
    #[must_use]
    pub fn is_valid(self) -> bool {
        SmallIE::default_for_typecode(self.0[0]).is_some()
    }

    #[must_use]
    pub fn terminator() -> Self {
        Self([0; size_of::<SmallIE>()])
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct IEDeserializationError;

impl TryFrom<IEBuf> for SmallIE {
    type Error = IEDeserializationError;
    fn try_from(value: IEBuf) -> Result<Self, Self::Error> {
        let mut new = Self::default_for_typecode(value.0[0]).ok_or(IEDeserializationError)?;
        let bytes = &value.0[1..];
        let target = new.as_mut_bytes();
        target.copy_from_slice(&bytes[..target.len()]);

        Ok(new)
    }
}

impl TryFrom<&IEBuf> for &SmallIE {
    type Error = IEDeserializationError;

    #[allow(clippy::ptr_as_ptr)]
    fn try_from(value: &IEBuf) -> Result<Self, Self::Error> {
        SmallIE::default_for_typecode(value.0[0]).ok_or(IEDeserializationError)?;

        let ptr = core::ptr::from_ref(value) as *const _;

        Ok(unsafe { &*ptr })
    }
}

impl TryFrom<&mut IEBuf> for &mut SmallIE {
    type Error = IEDeserializationError;

    #[allow(clippy::ptr_as_ptr)]
    fn try_from(value: &mut IEBuf) -> Result<Self, Self::Error> {
        SmallIE::default_for_typecode(value.0[0]).ok_or(IEDeserializationError)?;

        let ptr = core::ptr::from_mut(value) as *mut _;

        Ok(unsafe { &mut *ptr })
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
