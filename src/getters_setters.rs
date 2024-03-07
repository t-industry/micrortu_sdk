use core::cell::Cell;

use ie_base::{IEBuf, IEDeserializationError, SmallIE};
use wasm_global_shared_data::ParseError;

use crate::wasm_unwrap;

#[derive(Debug)]
pub struct GetSingle<'a>(&'a IEBuf);
#[derive(Debug)]
pub struct GetSingleOptional<'a>(Option<&'a IEBuf>);
#[derive(Debug)]
pub struct GetMultiple<'a>(&'a [IEBuf]);
#[derive(Debug)]
pub struct GetMultipleOptional<'a>(&'a [IEBuf]);

#[derive(Debug, Clone, Copy)]
pub struct Dirty<'a> {
    bits: &'a Cell<u64>,
    starting: usize,
}

impl<'a> Dirty<'a> {
    #[doc(hidden)]
    pub fn new(bits: &'a Cell<u64>, starting: usize) -> Self {
        Self { bits, starting }
    }
    fn set(self, index: usize) {
        self.bits
            .set(self.bits.get() | (1 << (index + self.starting)));
    }
}

#[derive(Debug)]
pub struct SetSingle<'a>(&'a mut IEBuf, Dirty<'a>);
#[derive(Debug)]
pub struct SetSingleOptional<'a>(Option<&'a mut IEBuf>, Dirty<'a>);
#[derive(Debug)]
pub struct SetMultiple<'a>(&'a mut [IEBuf], Dirty<'a>);
#[derive(Debug)]
pub struct SetMultipleOptional<'a>(&'a mut [IEBuf], Dirty<'a>);

#[derive(Debug)]
pub struct GetSetSingle<'a>(&'a mut IEBuf, Dirty<'a>);
#[derive(Debug)]
pub struct GetSetSingleOptional<'a>(Option<&'a mut IEBuf>, Dirty<'a>);
#[derive(Debug)]
pub struct GetSetMultiple<'a>(&'a mut [IEBuf], Dirty<'a>);
#[derive(Debug)]
pub struct GetSetMultipleOptional<'a>(&'a mut [IEBuf], Dirty<'a>);

impl<'a> GetSingle<'a> {
    /// # Errors
    /// Returns `Err` if cannot be converted to `SmallIE`.
    pub fn get(&self) -> Result<SmallIE, IEDeserializationError> {
        (*self.0).try_into()
    }
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn new(source: &'a [IEBuf], _dirty: Dirty<'a>) -> Self {
        Self(wasm_unwrap(source.first()))
    }
}

impl<'a> GetSingleOptional<'a> {
    /// # Errors
    /// Returns `Err` if cannot be converted to `SmallIE`.
    pub fn get(&self) -> Result<Option<SmallIE>, IEDeserializationError> {
        match self.0 {
            Some(&iebuf) => Ok(Some(iebuf.try_into()?)),
            None => Ok(None),
        }
    }
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn new(source: &'a [IEBuf], _dirty: Dirty<'a>) -> Self {
        Self(source.first())
    }
}

impl<'a> GetMultiple<'a> {
    pub fn get(&self) -> impl Iterator<Item = Result<SmallIE, IEDeserializationError>> + '_ {
        Self::get_inner(self.0)
    }
    fn get_inner(
        this: &[IEBuf],
    ) -> impl Iterator<Item = Result<SmallIE, IEDeserializationError>> + '_ {
        this.iter().map(|&iebuf| iebuf.try_into())
    }
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn new(source: &'a [IEBuf], _dirty: Dirty<'a>) -> Self {
        Self(source)
    }
}

impl<'a> GetMultipleOptional<'a> {
    #[must_use]
    pub fn get(
        &self,
    ) -> Option<impl Iterator<Item = Result<SmallIE, IEDeserializationError>> + '_> {
        Self::get_inner(self.0)
    }
    fn get_inner(
        this: &[IEBuf],
    ) -> Option<impl Iterator<Item = Result<SmallIE, IEDeserializationError>> + '_> {
        if this.is_empty() {
            return None;
        }
        Some(this.iter().map(|&iebuf| iebuf.try_into()))
    }
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn new(source: &'a [IEBuf], _dirty: Dirty<'a>) -> Self {
        Self(source)
    }
}

impl<'a> SetSingle<'a> {
    pub fn set(&mut self, value: SmallIE) {
        *self.0 = value.into();
        self.1.set(0);
    }
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn new(source: &'a mut [IEBuf], dirty: Dirty<'a>) -> Self {
        Self(wasm_unwrap(source.first_mut()), dirty)
    }
}

impl<'a> SetSingleOptional<'a> {
    pub fn set(&mut self, value: SmallIE) {
        if let Some(iebuf) = self.0.as_deref_mut() {
            *iebuf = value.into();
            self.1.set(0);
        }
    }
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn new(source: &'a mut [IEBuf], dirty: Dirty<'a>) -> Self {
        Self(source.first_mut(), dirty)
    }
}

impl<'a> SetMultiple<'a> {
    pub fn set(&mut self, values: impl IntoIterator<Item = SmallIE>) {
        for (i, (iebuf, value)) in self.0.iter_mut().zip(values).enumerate() {
            *iebuf = value.into();
            self.1.set(i);
        }
    }
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn new(source: &'a mut [IEBuf], dirty: Dirty<'a>) -> Self {
        Self(source, dirty)
    }
}

impl<'a> SetMultipleOptional<'a> {
    pub fn set(&mut self, values: impl IntoIterator<Item = SmallIE>) {
        for (i, (iebuf, value)) in self.0.iter_mut().zip(values).enumerate() {
            *iebuf = value.into();
            self.1.set(i);
        }
    }
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn new(source: &'a mut [IEBuf], dirty: Dirty<'a>) -> Self {
        Self(source, dirty)
    }
}

impl<'a> GetSetSingle<'a> {
    /// # Errors
    /// Returns `Err` if cannot be converted to `SmallIE`.
    pub fn get(&self) -> Result<SmallIE, IEDeserializationError> {
        GetSingle(self.0).get()
    }
    pub fn set(&mut self, value: SmallIE) {
        SetSingle(self.0, self.1).set(value);
    }
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn new(source: &'a mut [IEBuf], dirty: Dirty<'a>) -> Self {
        Self(wasm_unwrap(source.first_mut()), dirty)
    }
}

impl<'a> GetSetSingleOptional<'a> {
    /// # Errors
    /// Returns `Err` if cannot be converted to `SmallIE`.
    pub fn get(&self) -> Result<Option<SmallIE>, IEDeserializationError> {
        GetSingleOptional(self.0.as_deref()).get()
    }
    pub fn set(&mut self, value: SmallIE) {
        SetSingleOptional(self.0.as_deref_mut(), self.1).set(value);
    }
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn new(source: &'a mut [IEBuf], dirty: Dirty<'a>) -> Self {
        Self(source.first_mut(), dirty)
    }
}

impl<'a> GetSetMultiple<'a> {
    pub fn get(&self) -> impl Iterator<Item = Result<SmallIE, IEDeserializationError>> + '_ {
        GetMultiple::get_inner(self.0)
    }
    pub fn set(&mut self, values: impl IntoIterator<Item = SmallIE>) {
        SetMultiple(self.0, self.1).set(values);
    }
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn new(source: &'a mut [IEBuf], dirty: Dirty<'a>) -> Self {
        Self(source, dirty)
    }
}

impl<'a> GetSetMultipleOptional<'a> {
    #[must_use]
    pub fn get(
        &self,
    ) -> Option<impl Iterator<Item = Result<SmallIE, IEDeserializationError>> + '_> {
        GetMultipleOptional::get_inner(self.0)
    }
    pub fn set(&mut self, values: impl IntoIterator<Item = SmallIE>) {
        SetMultipleOptional(self.0, self.1).set(values);
    }
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn new(source: &'a mut [IEBuf], dirty: Dirty<'a>) -> Self {
        Self(source, dirty)
    }
}

pub fn parse_port<'a, 'd>(
    source: &'a [IEBuf],
    cursor: &'a usize,
    dirty: &'d Cell<u64>,
    is_optional: bool,
    min_size: u8,
    max_size: Option<u8>,
) -> Result<(usize, Dirty<'d>), ParseError> {
    let mut len = 0;
    if source.get(len).is_some() {
        let err = ParseError::NotTerminated;
        while source.get(len).ok_or(err)?.is_valid() {
            len = len.wrapping_add(1);
        }
    }
    let dirty = Dirty::new(dirty, *cursor);
    if len == 0 && is_optional {
        return Ok((0, dirty));
    }
    if len < min_size.into() {
        return Err(ParseError::NotEnoughData);
    }
    if max_size.map_or(false, |m: u8| len > m as usize) {
        return Err(ParseError::TooMuchData);
    }

    Ok((len, dirty))
}
