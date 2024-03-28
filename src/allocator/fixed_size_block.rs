use core::{mem, ptr};
use core::alloc::{GlobalAlloc, Layout};

use crate::allocator::Locked;

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

struct ListNode {
    next: Option<&'static mut ListNode>,
}

pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator
            .init(heap_start as *mut u8, heap_size);
    }
}

impl FixedSizeBlockAllocator {
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        let result = self.fallback_allocator.allocate_first_fit(layout);
        match result {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }

    fn list_index(&self, layout: Layout) -> Option<usize> {
        let required_block_size = layout.size().max(layout.align());
        BLOCK_SIZES
            .iter()
            .position(|&size| size >= required_block_size)
    }
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut alloc = self.lock();

        match alloc.list_index(layout) {
            // If none, then no size exists
            None => alloc.fallback_alloc(layout),
            Some(index) => match alloc.list_heads[index].take() {
                // If none, then no blocks left, so make one.
                None => {
                    let block_size = BLOCK_SIZES[index];
                    let block_align = block_size;
                    let layout = Layout::from_size_align(block_size, block_align).unwrap();
                    alloc.fallback_alloc(layout)
                }
                Some(node) => {
                    alloc.list_heads[index] = node.next.take();
                    node as *mut ListNode as *mut u8
                }
            },
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut alloc = self.lock();
        match alloc.list_index(layout) {
            None => {
                let ptr = ptr::NonNull::new(ptr).unwrap();
                alloc.fallback_allocator.deallocate(ptr, layout);
            }

            Some(index) => {
                let new_node = ListNode {
                    next: alloc.list_heads[index].take(),
                };
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                alloc.list_heads[index] = Some(&mut *new_node_ptr)
            }
        }
    }
}
