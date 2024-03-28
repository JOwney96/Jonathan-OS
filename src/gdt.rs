use lazy_static::lazy_static;
use x86_64::registers::segmentation::CS;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const STACK_SIZE: usize = 4096 * 5;

lazy_static! {
    // TSS is a relic of the past that contains:
        // The stack pointer addresses for each privilege level.
        // Pointer Addresses for the Interrupt Stack Table.
            // We are using the IST to give the double fault handler a known good stack.
        // Offset Address of the IO permission bitmap.

    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // TODO: Replace this nasty static mut with a proper stack
            // This requires memory management
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe{&STACK});
            let stack_end = stack_start + STACK_SIZE;

            stack_end
        };

        tss
    };
}

lazy_static! {
    // GDT is a relic of the past as well that contains segments such as the TSS
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::Segment;
    use x86_64::instructions::tables::load_tss;

    // Load the new GDT
    GDT.0.load();

    // Reload the code segment
    // Load the new tss struct
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
