use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

use crate::allocator::{align_up, Locked};

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocator_counter: usize,
}

impl BumpAllocator {
    /// Create a default bump allocator
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocator_counter: 0,
        }
    }

    /// Initializes the bump allocator
    ///
    /// This function is unsafe because the caller must ensure the given
    /// memory range is valid. Also, this method should only be called once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start.saturating_add(heap_size);
        self.next = self.heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();

        // Get the start by taking next from the bump and aligning it up to match the layout
        // alignment
        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(num) => num,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            return ptr::null_mut();
        };

        bump.next = alloc_end;
        bump.allocator_counter += 1;
        alloc_start as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock();

        if bump.allocator_counter > 0 {
            bump.allocator_counter -= 1;
        }

        if bump.allocator_counter == 0 {
            bump.next = bump.heap_start;
        }
    }
}
