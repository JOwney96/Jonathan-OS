#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(jonathan_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::panic::PanicInfo;

use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

use jonathan_os::{allocator, memory};
use jonathan_os::allocator::HEAP_SIZE;
use jonathan_os::memory::BootInfoFrameAllocator;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    jonathan_os::init();
    let phys_memory_offset = boot_info.physical_memory_offset;
    let mut mapper = unsafe { memory::init(VirtAddr::new(phys_memory_offset)) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap allocation failed");

    test_main();
    jonathan_os::hlt_loop();
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    jonathan_os::test_panic_handler(info)
}

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn many_boxes_long_lived() {
    let long_lived_box = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(i, *x);
    }

    assert_eq!(*long_lived_box, 1);
}
