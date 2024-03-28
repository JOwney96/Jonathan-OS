// Main file which has the main starting point for the OS

#![no_std] // Don't link the standard lib
#![no_main] // Because we are declaring the _start having a main is useless
#![feature(custom_test_frameworks)]
#![test_runner(jonathan_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};
use x86_64::VirtAddr;

use jonathan_os::{allocator, hlt_loop, memory, println};

//  ---Main Functions---

// Bootloader macro to force function signature correctness.
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World");

    jonathan_os::init();
    let virtual_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(virtual_memory_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    println!(
        "reference count is {} now",
        Rc::strong_count(&cloned_reference)
    );

    println!("It Didn't Crash!");

    // Above, we set the test auto-generated function to be test_main.
    // Here we call it with the cfg(test) added so if we don't call cargo test,
    // this function call is ignored.
    #[cfg(test)]
    test_main();

    hlt_loop()
}

// Create the panic handler needed by the Rust compiler.
// This panic handler is for non-test
#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    println!("{}", info);

    hlt_loop()
}

// This panic handler is for tests
#[cfg(test)]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    jonathan_os::test_panic_handler(info);
}
