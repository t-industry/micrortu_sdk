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

    pub fn alloc<T>(&self, value: T) -> &'static mut T {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();

        let allocated = self.alloc_inner(size, align);
        let ptr = ptr::from_mut::<[u8]>(allocated).cast::<T>();

        // Initialize the value, not dropping the previous one
        unsafe { ptr.write(value) };

        // SAFETY: allocated is properly aligned and has the correct size,
        //         memory is uniquely referenced
        unsafe { &mut *ptr }
    }

    fn get_refcell(&self) -> &RefCell<&'static mut [u8]> {
        // Transmute from `[usize; 2]` to `&mut [u8]`
        // array's first element is always non zero
        unsafe { &*ptr::from_ref(&self.buffer).cast() }
    }

    fn alloc_inner(&self, size: usize, align: usize) -> &'static mut [u8] {
        let mut borrow_mut = self.get_refcell().borrow_mut();
        let current_address = borrow_mut.as_ptr() as usize;
        let to_cut = ((current_address + align - 1) & !(align - 1)) - current_address;

        if to_cut + size > borrow_mut.len() {
            log::error!("BumpAllocator out of memory");
            panic!("BumpAllocator out of memory");
        }

        let buffer = mem::take(&mut *borrow_mut);
        let buffer = &mut buffer[to_cut..];
        let (allocated, rest) = buffer.split_at_mut(size);
        *borrow_mut = rest;

        allocated
    }
}

impl Default for BumpAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use static_cell::StaticCell;

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
