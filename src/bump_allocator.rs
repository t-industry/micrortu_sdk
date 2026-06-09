use core::{cell::RefCell, mem, ptr};

#[repr(C)]
union SliceOrEmpty {
    #[allow(dead_code)]
    slice: &'static mut [u8],
    empty: [usize; 2],
}

pub struct BumpAllocator {
    buffer: RefCell<SliceOrEmpty>,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct AllocError;

// Safety: BumpAllocator is not Sync, but wasm is single-threaded
#[cfg(any(target_arch = "wasm32", feature = "micrortu_sdk_internal"))]
unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    #[must_use]
    pub const fn new() -> Self {
        // Non-zero pointer
        let buffer = RefCell::new(SliceOrEmpty { empty: [1, 0] });
        Self { buffer }
    }

    pub fn replace_buffer(&self, buffer: &'static mut [u8]) -> &'static mut [u8] {
        let mut borrow_mut = self.get_refcell().borrow_mut();
        mem::replace(&mut *borrow_mut, buffer)
    }

    #[inline(always)]
    pub fn alloc<'a, T>(&self, value: T) -> &'a mut T {
        match self.try_alloc(value) {
            Ok(buffer) => buffer,
            Err(e) => {
                log::error!("{e}");
                panic!("{e}");
            }
        }
    }

    #[inline(always)]
    pub fn try_alloc<'a, T>(&self, value: T) -> Result<&'a mut T, AllocError> {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();

        let allocated = self.try_alloc_inner(size, align)?;
        let ptr = ptr::from_mut::<[u8]>(allocated).cast::<mem::MaybeUninit<T>>();

        // SAFETY: `ptr` is properly aligned and has the correct size,
        //         memory is uniquely referenced
        let maybe_uninit = unsafe { &mut *ptr };

        Ok(maybe_uninit.write(value))
    }

    fn get_refcell(&self) -> &RefCell<&'static mut [u8]> {
        // Transmute from `[usize; 2]` to `&mut [u8]`
        // array's first element is always non zero
        unsafe { &*ptr::from_ref(&self.buffer).cast() }
    }

    fn try_alloc_inner(&self, size: usize, align: usize) -> Result<&'static mut [u8], AllocError> {
        let mut borrow_mut = self.get_refcell().borrow_mut();
        let current_address = borrow_mut.as_ptr() as usize;
        let to_cut = ((current_address + align - 1) & !(align - 1)) - current_address;

        if to_cut + size > borrow_mut.len() {
            return Err(AllocError);
        }

        let buffer = mem::take(&mut *borrow_mut);
        let buffer = &mut buffer[to_cut..];
        let (allocated, rest) = buffer.split_at_mut(size);
        *borrow_mut = rest;

        Ok(allocated)
    }
}

impl Default for BumpAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for AllocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BumpAllocator out of memory")
    }
}
impl core::fmt::Display for AllocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BumpAllocator out of memory")
    }
}
impl core::error::Error for AllocError {}

#[cfg(test)]
mod test {
    use super::*;
    use static_cell::StaticCell;

    #[test]
    fn cycle_buffers() {
        static BUFFER1: StaticCell<[u8; 100]> = StaticCell::new();
        static BUFFER2: StaticCell<[u8; 100]> = StaticCell::new();
        let buffer1 = BUFFER1.init([0; 100]);
        let buffer2 = BUFFER2.init([0; 100]);
        let p1 = buffer1.as_ptr() as usize;
        let p2 = buffer2.as_ptr() as usize;
        let allocator = BumpAllocator::new();
        assert!(allocator.replace_buffer(buffer1).is_empty());
        assert!(allocator.replace_buffer(buffer2).as_ptr() as usize == p1);
        assert!(allocator.replace_buffer(&mut []).as_ptr() as usize == p2);
        assert!(allocator.replace_buffer(&mut []).is_empty());
    }

    #[test]
    fn test_bump_allocator() {
        static BUFFER: StaticCell<[u8; 100]> = StaticCell::new();
        let buffer = BUFFER.init([0; 100]);
        let allocator = BumpAllocator::new();
        assert!(allocator.replace_buffer(buffer).is_empty());
        let a = allocator.alloc::<u32>(1);
        let b = allocator.alloc::<u8>(2);
        let c = allocator.alloc::<u32>(3);
        assert_eq!(*a, 1);
        assert_eq!(*b, 2);
        assert_eq!(*c, 3);
    }

    #[test]
    fn test_bump_allocator_uninit() {
        static BUFFER: StaticCell<[u8; 100]> = StaticCell::new();
        let buffer = BUFFER.init([0; 100]);
        let allocator = BumpAllocator::new();
        assert!(allocator.replace_buffer(buffer).is_empty());
        let a = allocator.alloc::<u32>(1);
        let b = allocator.alloc::<&'static [u8]>(&[]);
        let c = allocator.alloc::<u32>(3);
        assert_eq!(*a, 1);
        assert_eq!(*b, &[]);
        assert_eq!(*c, 3);
    }

    #[test]
    fn test_bump_allocator_with_bad_align() {
        static BUFFER: StaticCell<[u8; 100]> = StaticCell::new();
        let buffer = BUFFER.init([0; 100]);
        let allocator = BumpAllocator::new();
        assert!(allocator.replace_buffer(&mut buffer[1..]).is_empty());
        let a = allocator.alloc::<u32>(1);
        let b = allocator.alloc::<u8>(2);
        let c = allocator.alloc::<u32>(3);
        assert_eq!(*a, 1);
        assert_eq!(*b, 2);
        assert_eq!(*c, 3);
    }

    #[test]
    #[should_panic(expected = "BumpAllocator out of memory")]
    fn test_bump_allocator_out_of_memory() {
        static BUFFER: StaticCell<[u8; 100]> = StaticCell::new();
        let buffer = BUFFER.init([0; 100]);
        let allocator = BumpAllocator::new();
        assert!(allocator.replace_buffer(buffer).is_empty());
        let a = allocator.alloc::<u32>(1);
        let b = allocator.alloc::<[u8; 100]>([1; 100]);
        core::hint::black_box(&a);
        core::hint::black_box(&b);
    }
}
