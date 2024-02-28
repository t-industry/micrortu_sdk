use core::{cell::RefCell, mem};

pub struct BumpAllocator {
    buffer: RefCell<&'static mut [u8]>,
}

impl BumpAllocator {
    pub fn new(buffer: &'static mut [u8]) -> Self {
        let buffer = RefCell::new(buffer);
        Self { buffer }
    }

    pub fn alloc<T>(&self) -> &'static mut T {
        let size = core::mem::size_of::<T>();
        let align = core::mem::align_of::<T>();

        let allocated = self.alloc_inner(size, align);
        let ptr = core::ptr::from_mut::<[u8]>(allocated).cast::<T>();

        unsafe { &mut *ptr }
    }

    fn alloc_inner(&self, size: usize, align: usize) -> &'static mut [u8] {
        let mut borrow_mut = self.buffer.borrow_mut();
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

#[cfg(test)]
mod test {
    use super::*;
    use static_cell::StaticCell;

    #[test]
    fn test_bump_allocator() {
        static BUFFER: StaticCell<[u8; 100]> = StaticCell::new();
        let buffer = BUFFER.init([0; 100]);
        let allocator = BumpAllocator::new(buffer);
        let a = allocator.alloc::<u32>();
        let b = allocator.alloc::<u8>();
        let c = allocator.alloc::<u32>();
        *a = 1;
        *b = 2;
        *c = 3;
        assert_eq!(*a, 1);
        assert_eq!(*b, 2);
        assert_eq!(*c, 3);
    }

    #[test]
    fn test_bump_allocator_with_bad_align() {
        static BUFFER: StaticCell<[u8; 100]> = StaticCell::new();
        let buffer = BUFFER.init([0; 100]);
        let allocator = BumpAllocator::new(&mut buffer[1..]);
        let a = allocator.alloc::<u32>();
        let b = allocator.alloc::<u8>();
        let c = allocator.alloc::<u32>();
        *a = 1;
        *b = 2;
        *c = 3;
        assert_eq!(*a, 1);
        assert_eq!(*b, 2);
        assert_eq!(*c, 3);
    }

    #[test]
    #[should_panic(expected = "BumpAllocator out of memory")]
    fn test_bump_allocator_out_of_memory() {
        static BUFFER: StaticCell<[u8; 100]> = StaticCell::new();
        let buffer = BUFFER.init([0; 100]);
        let allocator = BumpAllocator::new(buffer);
        let a = allocator.alloc::<u32>();
        let b = allocator.alloc::<[u8; 100]>();
        core::hint::black_box(&a);
        core::hint::black_box(&b);
    }
}
