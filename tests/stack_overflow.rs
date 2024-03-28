#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use jonathan_os::gdt::DOUBLE_FAULT_IST_INDEX;
use jonathan_os::{exit_qemu, serial_print, serial_println, QemuExitCode};

#[no_mangle]
extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    jonathan_os::gdt::init();
    init_test_idt();

    stack_overflow();

    panic!("Execution continued after stack overflow")
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _interrupt_stack_frame: InterruptStackFrame,
    _err_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);

    loop {}
}

fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read(); // prevent tail recursion optimizations
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    jonathan_os::test_panic_handler(info);
}
