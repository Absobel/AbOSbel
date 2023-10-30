use core::arch::asm;

pub unsafe fn inb(port: u16) -> u8 {
    let data;
    asm!("in al, dx", in("dx") port, out("al") data);
    data
}

pub unsafe fn outb(port: u16, data: u8) {
    asm!("out dx, al", in("dx") port, in("al") data);
}

pub unsafe fn inw(port: u16) -> u16 {
    let data;
    asm!("in ax, dx", in("dx") port, out("ax") data);
    data
}

pub unsafe fn outw(port: u16, data: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") data);
}

///////////////////////////////

pub const PS2_KEYBOARD_IN: u16 = 0x60;
pub const PS2_KEYBOARD_OUT: u16 = 0x64;
