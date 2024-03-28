// This file is for creating shared links.
// Add to mod or function to this file if you want it shared between other mods and tests

#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(const_mut_refs)]

extern crate alloc;

use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::{BootInfo, entry_point};

pub mod allocator;
pub mod apic;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga_buffer;

//  ---Init---

pub fn init() {
    interrupts::init_idt();
    gdt::init();
    unsafe {
        x86_64::instructions::interrupts::without_interrupts(|| {
            interrupts::PICS.lock().initialize();
        });
    }

    x86_64::instructions::interrupts::enable();
}

//  ---Misc Functions

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

//  ---QEMU Exit Components---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4); // We set this port in the cargo file
        port.write(exit_code as u32);
    }
}

//  ---Test Components---

pub trait Testable {
    fn run(&self) -> ();
}

// Testable trait that adds test name before function call and passed after call if passed
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);

    hlt_loop()
}

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo test`
#[cfg(test)]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    init();
    test_main();

    hlt_loop()
}

/// Panic handler for `cargo test`
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
