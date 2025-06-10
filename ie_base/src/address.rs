use core::str::FromStr;

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};
#[cfg(feature = "rkyv")]
use {
    bytecheck::CheckBytes,
    rkyv::{Archive, Portable, Serialize},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
#[repr(C)]
pub struct CA(pub u8, pub u8);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)] //
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout)] //
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Portable, CheckBytes))] //
#[cfg_attr(feature = "rkyv", rkyv(as = Self))]
#[repr(C)]
pub struct IOA(pub u8, pub u8, pub u8);

#[derive(Debug)]
pub struct ParseError;

fn parse_dot_separated<const N: usize>(src: &str) -> Result<[u8; N], ParseError> {
    let mut res = [0u8; N];

    if src.split('.').count() != N {
        return Err(ParseError);
    }

    for (dst, src) in res.iter_mut().zip(src.split('.')) {
        *dst = src.parse().map_err(|_| ParseError)?;
    }

    Ok(res)
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

impl FromStr for CA {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [a, b] = parse_dot_separated(s)?;
        Ok(Self(a, b))
    }
}

impl FromStr for IOA {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [a, b, c] = parse_dot_separated(s)?;
        Ok(Self(a, b, c))
    }
}

impl core::fmt::Display for IOA {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "IOA({}.{}.{})", self.0, self.1, self.2)
    }
}

impl core::fmt::Display for CA {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CA({}.{})", self.0, self.1)
    }
}
