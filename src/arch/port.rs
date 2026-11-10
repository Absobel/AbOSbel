use core::{arch::asm, fmt::Debug};

#[derive(Debug, Clone, Copy)]
pub struct Port<T> {
    port: u16,
    _phantom: core::marker::PhantomData<T>,
}

impl<T> Port<T> {
    #[must_use]
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl Port<u8> {
    #[must_use]
    pub unsafe fn inb(self) -> u8 {
        let data;
        unsafe { asm!("in al, dx", in("dx") self.port, out("al") data) };
        data
    }

    pub unsafe fn outb(self, data: u8) {
        unsafe { asm!("out dx, al", in("dx") self.port, in("al") data) };
    }
}

#[expect(dead_code)]
impl Port<u16> {
    #[must_use]
    pub unsafe fn inw(self) -> u16 {
        let data;
        unsafe { asm!("in ax, dx", in("dx") self.port, out("ax") data) };
        data
    }

    pub unsafe fn outw(self, data: u16) {
        unsafe { asm!("out dx, ax", in("dx") self.port, in("ax") data) };
    }
}

#[expect(dead_code)]
impl Port<u32> {
    #[must_use]
    pub unsafe fn inl(self) -> u32 {
        let data;
        unsafe { asm!("in eax, dx", in("dx") self.port, out("eax") data) };
        data
    }

    pub unsafe fn outl(self, data: u32) {
        unsafe { asm!("out dx, eax", in("dx") self.port, in("eax") data) };
    }
}

/// Function to wait for an I/O operation to complete when the hardware does not provide a way to do so
pub fn wait() {
    const DUMMY_PORT: Port<u8> = Port::new(0x80);
    unsafe {
        DUMMY_PORT.outb(0);
    }
}
