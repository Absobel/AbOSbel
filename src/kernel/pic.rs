use core::sync::atomic::AtomicBool;

use crate::kernel::{
    io::{self, Port},
    x86,
};

const MASTER_COMMAND: Port<u8> = Port::new(0x20);
const MASTER_DATA: Port<u8> = Port::new(0x21);
const SLAVE_COMMAND: Port<u8> = Port::new(0xA0);
const SLAVE_DATA: Port<u8> = Port::new(0xA1);

/// Initialization Bit
const ICW1_INIT: u8 = 0x10;
/// Indicate that the ICW4 will be present
const ICW1_ICW4: u8 = 0x01;
/// Offset of the master PIC's interrupt vector
const MASTER_OFFSET: u8 = 0x20;
/// Offset of the slave PIC's interrupt vector
const SLAVE_OFFSET: u8 = 0x28;
/// Number of the IRQ line allowing cascading from the slave PIC to the master PIC
const CASCADE_IRQ: u8 = 2;
/// 8086/88 (MCS-80/85) mode
const ICW4_8086: u8 = 0x01;

#[derive(Debug, Clone, Copy)]
enum Interrupts {
    Timer = 0,
    Keyboard = 1,
    Cascade = 2,
    // ...
    // Mouse = 12,
}

pub fn init() {
    unsafe {
        // Init bits
        MASTER_COMMAND.outb(ICW1_INIT | ICW1_ICW4);
        io::wait();
        SLAVE_COMMAND.outb(ICW1_INIT | ICW1_ICW4);
        io::wait();
        // Vector offsets
        MASTER_DATA.outb(MASTER_OFFSET);
        io::wait();
        SLAVE_DATA.outb(SLAVE_OFFSET);
        io::wait();
        // Cascade configuration
        MASTER_DATA.outb(1 << CASCADE_IRQ);
        io::wait();
        SLAVE_DATA.outb(CASCADE_IRQ);
        io::wait();
        // Use the 8086/88 mode
        MASTER_DATA.outb(ICW4_8086);
        io::wait();
        SLAVE_DATA.outb(ICW4_8086);
        io::wait();
        // Mask all interrupts
        MASTER_DATA.outb(0xFF);
        io::wait();
        SLAVE_DATA.outb(0xFF);
        io::wait();
    }

    // mask_irq(Mask::Clear, Interrupts::Timer);
    mask_irq(Mask::Clear, Interrupts::Cascade);
    mask_irq(Mask::Clear, Interrupts::Keyboard);

    // enable interrupts
    // mask_irq(Mask::Set, Interrupts::Timer);
    x86::enable_interrupts();
}

#[expect(dead_code)]
#[derive(Debug, Clone, Copy)]
enum Mask {
    Clear,
    Set,
}

fn mask_irq(mode: Mask, irq: Interrupts) {
    let irq = irq as u8;
    let (port, mask) = if irq < 8 {
        (MASTER_DATA, 1 << irq)
    } else {
        (SLAVE_DATA, 1 << (irq - 8))
    };

    unsafe {
        let current_mask = port.inb();
        match mode {
            Mask::Clear => port.outb(current_mask & !mask),
            Mask::Set => port.outb(current_mask | mask),
        }
        io::wait();
    }
}

fn send_eoi(irq: Interrupts) {
    const EOI: u8 = 0x20;

    let irq = irq as u8;
    if irq >= 8 {
        unsafe { SLAVE_COMMAND.outb(EOI) };
    }
    unsafe { MASTER_COMMAND.outb(EOI) };
}

// Handlers

pub static WAITING_FOR_EXTENDED: AtomicBool = AtomicBool::new(false);


macro_rules! pic_interrupt {
    ($name:ident, $txt:literal, $irq:expr) => {
        pub extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            println!("[PIC] Interrupt: {} at address {:#x}", $txt, stack_frame.rip);
            crate::kernel::pic::send_eoi($irq);
        }
    };
}

#[expect(clippy::wildcard_imports)]
pub mod handlers {
    use core::sync::atomic::Ordering;

use crate::{
        kernel::{interrupts::InterruptStackFrame, pic::*}, print, println,
    };

    pic_interrupt!(timer, "TIMER", Interrupts::Timer);
    
    pub extern "x86-interrupt" fn keyboard(_stack_frame: InterruptStackFrame) {
        let scancode: u8 = unsafe { PS2_DATA_PORT.inb() };
        if scancode == EXTENDED_SCANCODE {
            WAITING_FOR_EXTENDED.store(true, Ordering::SeqCst);
        } else {
            let key_code = scancode & !(1 << 7); // Clear the release bit
            let release_bit = (scancode & (1 << 7)) != 0;
            let is_extended = WAITING_FOR_EXTENDED.load(Ordering::SeqCst);
            let key = if is_extended {
                WAITING_FOR_EXTENDED.store(false, Ordering::SeqCst);
                SCAN_CODE_TABLE_EXTENDED[key_code as usize]
            } else {
                SCAN_CODE_TABLE[key_code as usize]
            };

            if let Some(key) = key {
                print!("[PIC] Key: {:?} ", key);
            } else {
                print!("[PIC] Unknown key (scancode: {:#x}) ", scancode);
            }
            println!("{}", if release_bit { "Released" } else { "Pressed" });
        }
        send_eoi(Interrupts::Keyboard);
    }
}

// PS/2 Controller

const PS2_DATA_PORT: Port<u8> = Port::new(0x60);
const PS2_COMMAND_PORT: Port<u8> = Port::new(0x64);

struct KeyEvent {
    key: Key,
    state: KeyState,
    state_machine: KeyStateMachine,
}

#[derive(Debug, Clone, Copy)]
enum KeyState {
    Released,
    Pressed,
}

#[derive(Debug, Clone, Copy)]
struct KeyStateMachine(u128);

impl KeyStateMachine {
    const fn new() -> Self {
        Self(0)
    }

    const fn set_key_state(&mut self, key: Key, state: KeyState) {
        self.0 = match state {
            KeyState::Pressed => self.0 | (1 << key as u128),
            KeyState::Released => self.0 & !(1 << key as u128),
        };
    }

    const fn is_key_pressed(&self, key: Key) -> bool {
        (self.0 & (1 << key as u128)) != 0
    }
}

// Keys, add 0x80 to the scan code for key release events

#[expect(clippy::enum_glob_use)]
#[rustfmt::skip]
pub const SCAN_CODE_TABLE: [Option<Key>; 128] = {
    use Key::*;
    [
        /* 0x00 */ None,                 Some(Escape),         Some(One),            Some(Two),
        /* 0x04 */ Some(Three),          Some(Four),           Some(Five),           Some(Six),
        /* 0x08 */ Some(Seven),          Some(Eight),          Some(Nine),           Some(Zero),
        /* 0x0C */ Some(Minus),          Some(Equal),          Some(Backspace),      Some(Tab),
        /* 0x10 */ Some(Q),              Some(W),              Some(E),              Some(R),
        /* 0x14 */ Some(T),              Some(Y),              Some(U),              Some(I),
        /* 0x18 */ Some(O),              Some(P),              Some(LeftBracket),    Some(RightBracket),
        /* 0x1C */ Some(Enter),          Some(LeftCtrl),       Some(A),              Some(S),
        /* 0x20 */ Some(D),              Some(F),              Some(G),              Some(H),
        /* 0x24 */ Some(J),              Some(K),              Some(L),              Some(Semicolon),
        /* 0x28 */ Some(Apostrophe),     Some(Backtick),       Some(LeftShift),      Some(Backslash),
        /* 0x2C */ Some(Z),              Some(X),              Some(C),              Some(V),
        /* 0x30 */ Some(B),              Some(N),              Some(M),              Some(Comma),
        /* 0x34 */ Some(Period),         Some(Slash),          Some(RightShift),     Some(KeypadMultiply),
        /* 0x38 */ Some(LeftAlt),        Some(Space),          Some(CapsLock),       Some(F1),
        /* 0x3C */ Some(F2),             Some(F3),             Some(F4),             Some(F5),
        /* 0x40 */ Some(F6),             Some(F7),             Some(F8),             Some(F9),
        /* 0x44 */ Some(F10),            Some(NumLock),        Some(ScrollLock),     Some(Keypad7),
        /* 0x48 */ Some(Keypad8),        Some(Keypad9),        Some(KeypadMinus),    Some(Keypad4),
        /* 0x4C */ Some(Keypad5),        Some(Keypad6),        Some(KeypadPlus),     Some(Keypad1),
        /* 0x50 */ Some(Keypad2),        Some(Keypad3),        Some(Keypad0),        Some(KeypadPeriod),
        /* 0x54 */ None,                 None,                 Some(NonUsBackslash), Some(F11),
        /* 0x58 */ Some(F12),            None,                 None,                 None,
        /* 0x5C */ None,                 None,                 None,                 None,
        /* 0x60 */ None,                 None,                 None,                 None,
        /* 0x64 */ None,                 None,                 None,                 None,
        /* 0x68 */ None,                 None,                 None,                 None,
        /* 0x6C */ None,                 None,                 None,                 None,
        /* 0x70 */ None,                 None,                 None,                 None,
        /* 0x74 */ None,                 None,                 None,                 None,
        /* 0x78 */ None,                 None,                 None,                 None,
        /* 0x7C */ None,                 None,                 None,                 None,
    ]
};

pub const EXTENDED_SCANCODE: u8 = 0xE0;

#[expect(clippy::enum_glob_use)]
#[rustfmt::skip]
pub const SCAN_CODE_TABLE_EXTENDED: [Option<Key>; 128] = {
    use Key::*;
    [
        /* 0x00 */ None,                 None,                 None,                 None,
        /* 0x04 */ None,                 None,                 None,                 None,
        /* 0x08 */ None,                 None,                 None,                 None,
        /* 0x0C */ None,                 None,                 None,                 None,
        /* 0x10 */ None,                 None,                 None,                 None,
        /* 0x14 */ None,                 None,                 None,                 None,
        /* 0x18 */ None,                 None,                 None,                 None,
        /* 0x1C */ Some(KeypadEnter),    Some(RightCtrl),      None,                 None,
        /* 0x20 */ None,                 None,                 None,                 None,
        /* 0x24 */ None,                 None,                 None,                 None,
        /* 0x28 */ None,                 None,                 None,                 None,
        /* 0x2C */ None,                 None,                 None,                 None,
        /* 0x30 */ None,                 None,                 None,                 None,
        /* 0x34 */ None,                 Some(KeypadDivide),   None,                 Some(PrintScreen),
        /* 0x38 */ Some(RightAlt),       None,                 None,                 None,
        /* 0x3C */ None,                 None,                 None,                 None,
        /* 0x40 */ None,                 None,                 None,                 None,
        /* 0x44 */ None,                 None,                 None,                 Some(Home),
        /* 0x48 */ Some(Up),             Some(PageUp),         None,                 Some(Left),
        /* 0x4C */ None,                 Some(Right),          None,                 Some(End),
        /* 0x50 */ Some(Down),           Some(PageDown),       Some(Insert),         Some(Delete),
        /* 0x54 */ None,                 None,                 None,                 None,
        /* 0x58 */ None,                 None,                 None,                 Some(LeftSuper),
        /* 0x5C */ Some(RightSuper),     Some(Menu),           None,                 None,
        /* 0x60 */ None,                 None,                 None,                 None,
        /* 0x64 */ None,                 None,                 None,                 None,
        /* 0x68 */ None,                 None,                 None,                 None,
        /* 0x6C */ None,                 None,                 None,                 None,
        /* 0x70 */ None,                 None,                 None,                 None,
        /* 0x74 */ None,                 None,                 None,                 None,
        /* 0x78 */ None,                 None,                 None,                 None,
        /* 0x7C */ None,                 None,                 None,                 None,
    ]
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Escape,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Zero,
    Minus,
    Equal,
    Backspace,
    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    LeftBracket,
    RightBracket,
    Enter,
    LeftCtrl,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Semicolon,
    Apostrophe,
    Backtick,
    LeftShift,
    Backslash,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,
    Period,
    Slash,
    RightShift,
    KeypadMultiply,
    LeftAlt,
    Space,
    CapsLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    NumLock,
    ScrollLock,
    Keypad7,
    Keypad8,
    Keypad9,
    KeypadMinus,
    Keypad4,
    Keypad5,
    Keypad6,
    KeypadPlus,
    Keypad1,
    Keypad2,
    Keypad3,
    Keypad0,
    KeypadPeriod,
    NonUsBackslash,
    F11,
    F12,
    // Extended (E0)
    KeypadEnter,
    RightCtrl,
    KeypadDivide,
    PrintScreen,
    RightAlt,
    Home,
    Up,
    PageUp,
    Left,
    Right,
    End,
    Down,
    PageDown,
    Insert,
    Delete,
    LeftSuper,
    RightSuper,
    Menu,
    // E1 sequence, not in the tables
    Pause,
}
