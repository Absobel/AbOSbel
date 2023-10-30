use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::{gdt, print, println};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Software interruptions

        idt.divide_error.set_handler_fn(divide_by_zero_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt
            .set_handler_fn(non_maskable_interrupt_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded
            .set_handler_fn(bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available
            .set_handler_fn(device_not_available_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        idt.vmm_communication_exception.set_handler_fn(vmm_communication_handler);
        idt.security_exception.set_handler_fn(security_exception_handler);

        // Hardware interruptions

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

// IDT HANDLERS
// TODO : Add info to the panic message

extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn non_maskable_interrupt_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: NON MASKABLE INTERRUPT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
}

#[derive(Debug)]
enum DoubleFaultErrorCode {
    ItAlwaysZero,
    InvalidErrorCode,
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    let error_enum = match error_code {
        0 => DoubleFaultErrorCode::ItAlwaysZero,
        _ => DoubleFaultErrorCode::InvalidErrorCode,
    };
    panic!(
        "EXCEPTION: DOUBLE FAULT\nError Code: {:?}\nStack Frame: {:#?}",
        error_enum, stack_frame
    );
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
    panic!("EXCEPTION: INVALID TSS\n{:#?}", stack_frame);
}

#[derive(Debug)]
enum SegmentNotPresentErrorCode {
    SegmentSelectorIndex(u64),
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: SEGMENT NOT PRESENT\nError Code: {:?}\nStack Frame: {:#?}",
        SegmentNotPresentErrorCode::SegmentSelectorIndex(error_code),
        stack_frame
    );
}

#[derive(Debug)]
enum StackSegmentFaultErrorCode {
    SegmentAlreadyPresent,
    SegmentSelectorIndex(u64),
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    let error_enum = match error_code {
        0 => StackSegmentFaultErrorCode::SegmentAlreadyPresent,
        _ => StackSegmentFaultErrorCode::SegmentSelectorIndex(error_code),
    };
    panic!(
        "EXCEPTION: STACK SEGMENT FAULT\nError Code: {:?}\nStack Frame: {:#?}",
        error_enum, stack_frame
    );
}

#[derive(Debug)]
enum GeneralProtectionFaultErrorCode {
    NothingInteresting,
    SegmentSelectorIndex(u64),
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    let error_enum = match error_code {
        0 => GeneralProtectionFaultErrorCode::NothingInteresting,
        _ => GeneralProtectionFaultErrorCode::SegmentSelectorIndex(error_code),
    };
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\nError Code: {:?}\nStack Frame: {:#?}",
        error_enum, stack_frame
    );
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    panic!(
        "EXCEPTION: PAGE FAULT\nAccessed Address: {:?}\nError Code: {:?}\nStack Frame: {:#?}",
        Cr2::read(),
        error_code,
        stack_frame
    );
}

extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: X87 FLOATING POINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: ALIGNMENT CHECK\nError Code: {:?}\nStack Frame: {:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: VIRTUALIZATION\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn vmm_communication_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: VMM COMMUNICATION EXCEPTION\nError Code: {:?}\nStack Frame: {:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn security_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: SECURITY EXCEPTION\nError Code: {:?}\nStack Frame: {:#?}",
        error_code, stack_frame
    );
}

// HARDWARE IDT HANDLERS

extern "x86-interrupt" fn _timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // TODO : Do smth here ig
    print!(".");

    // TODO : Notify end interrupt
}

extern "x86-interrupt" fn _keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // TODO : UEFI keyboard support
}

// TESTS

#[cfg(test)]
mod tests {
    #[test_case]
    fn no_panic_breakpoint_exception() {
        x86_64::instructions::interrupts::int3();
    }
}
