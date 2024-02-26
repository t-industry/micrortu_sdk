use zerocopy::{AsBytes, FromBytes, FromZeroes};

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Default, AsBytes, FromZeroes, FromBytes)]
pub struct CA(pub u8, pub u8);

impl core::fmt::Display for CA {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CA({}.{})", self.0, self.1)
    }
}

impl CA {
    #[must_use]
    pub fn is_broadcast(&self) -> bool {
        self.0 == 0xFF && self.1 == 0xFF
    }

    #[must_use]
    pub fn matches(&self, filter: Self) -> bool {
        filter.is_broadcast() || *self == filter
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Default, AsBytes, FromZeroes, FromBytes)]
pub struct IOA(pub u8, pub u8, pub u8);

impl IOA {
    #[must_use]
    pub fn as_unstructured(&self) -> u32 {
        ((self.0 as u32) << 16) | ((self.1 as u32) << 8) | (self.2 as u32)
    }

    #[must_use]
    pub fn from_unstructured(u: u32) -> Self {
        Self(
            ((u & 0x00FF_0000) >> 16) as u8,
            ((u & 0x0000_FF00) >> 8) as u8,
            (u & 0x0000_00FF) as u8,
        )
    }

    #[must_use]
    pub fn inc(&self) -> Self {
        Self::from_unstructured(self.as_unstructured() + 1)
    }
}

impl core::fmt::Display for IOA {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "IOA({}.{}.{})", self.0, self.1, self.2)
    }
}
