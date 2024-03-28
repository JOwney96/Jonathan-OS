// Mod for handing interrupts and cpu exceptions
// This is using a lot of "magic."
// Think about switching to naked functions and creating this custom.
// Think about why I need to roll my own code.
// Question my sanity.
// Afterward will probably do it.

use lazy_static::lazy_static;
use pc_keyboard::{layouts, Keyboard, ScancodeSet1};
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions::hlt;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::{gdt, print, println};

//  ---IDT---

lazy_static! {
    // When an interrupt, of any kind including exceptions, occurs handler, or function, do we call?
    // This is the sole job of the IDT
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Here we are setting the handler for the breakpoint interrupt
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[PicInterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[PicInterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);

        unsafe {
            // Here we are setting the double fault handler
            // Note, The set_stack_index method is unsafe because the caller must ensure that the
                // used index is valid and not already used for another exception.
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

pub fn init_idt() {
    // Push the IDT into the CPU
    IDT.load();
}

//  ---Hardware Interrupt---

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum PicInterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl PicInterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

//  ---Handlers---

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// WARNING: This function does not have a guard page.
// Do NOT do anything stack intensive until this issue has been corrected.
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT\nERROR CODE: {}\n{:#?}",
        err_code, stack_frame
    );
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);

    loop {
        hlt();
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");

    unsafe {
        //apic::APIC.lock().end_of_interrupt();
        PICS.lock()
            .notify_end_of_interrupt(PicInterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(pc_keyboard::Keyboard::new(
                pc_keyboard::layouts::Us104Key,
                pc_keyboard::ScancodeSet1,
                pc_keyboard::HandleControl::Ignore
            ));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                pc_keyboard::DecodedKey::Unicode(character) => print!("{}", character),
                pc_keyboard::DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        //apic::APIC.lock().end_of_interrupt();
        PICS.lock()
            .notify_end_of_interrupt(PicInterruptIndex::Keyboard.as_u8());
    }
}
