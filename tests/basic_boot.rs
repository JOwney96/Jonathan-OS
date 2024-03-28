#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(jonathan_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use jonathan_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    jonathan_os::test_panic_handler(info);
    loop {}
}

fn test_runner(tests: &[&dyn Fn()]) {
    unimplemented!()
}

#[test_case]
fn test_println() {
    println!("test_print output");
}
