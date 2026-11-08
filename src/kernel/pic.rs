use core::{
    fmt::Display,
    sync::atomic::{AtomicBool, AtomicU128, Ordering},
};

use spin::Mutex;

use crate::{
    kernel::{
        io::{self, Port},
        x86::{self, without_interrupts},
    },
    print,
    utils::RingBuffer,
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

macro_rules! pic_interrupt {
    ($name:ident, $txt:literal, $irq:expr) => {
        pub extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            println!(
                "[PIC] Interrupt: {} at address {:#x}",
                $txt, stack_frame.rip
            );
            crate::kernel::pic::send_eoi($irq);
        }
    };
}

#[expect(clippy::wildcard_imports)]
pub mod handlers {
    use crate::{
        kernel::{interrupts::InterruptStackFrame, pic::*},
        println,
    };

    pic_interrupt!(timer, "TIMER", Interrupts::Timer);

    pub extern "x86-interrupt" fn keyboard(_stack_frame: InterruptStackFrame) {
        let scancode: u8 = unsafe { PS2_DATA_PORT.inb() };
        let key_event = KeyEvent::from_scancode(scancode);
        match key_event {
            Ok(event) => {
                // Discard the event if the ring buffer is full
                let _ = KEY_RING_BUFFER.lock().try_push(event);
            }
            Err(e @ KeyEventError::UnknownKey(_)) => {
                println!("[PIC] Error: {}", e);
            }
            _ => {}
        }
        send_eoi(Interrupts::Keyboard);
    }
}

// PS/2 Controller

const PS2_DATA_PORT: Port<u8> = Port::new(0x60);
#[expect(dead_code)]
const PS2_COMMAND_PORT: Port<u8> = Port::new(0x64);

pub static WAITING_FOR_EXTENDED: AtomicBool = AtomicBool::new(false);
static KEY_STATE_MACHINE: AtomicU128 = AtomicU128::new(KeyStateMachine::new().0);
pub static KEY_RING_BUFFER: Mutex<RingBuffer<KeyEvent, 16>> = Mutex::new(RingBuffer::new());

pub fn flush_key_events() {
    while let Some(event) = without_interrupts(|| KEY_RING_BUFFER.lock().try_pop()) {
        event.evaluate();
    }
}

#[derive(Clone, Copy)]
pub struct KeyEvent {
    key_state: KeyState,
    state_machine: KeyStateMachine,
}

impl KeyEvent {
    fn from_scancode(scancode: u8) -> Result<Self, KeyEventError> {
        let key_state = KeyState::from_code(scancode)?;
        let mut state_machine = KeyStateMachine(KEY_STATE_MACHINE.load(Ordering::SeqCst));
        let event = state_machine.set_key_state(key_state);
        KEY_STATE_MACHINE.store(state_machine.0, Ordering::SeqCst);
        Ok(event)
    }

    pub fn evaluate(&self) {
        // TODO: add a check for caps lock and numlock, which needs a separate state machine, or the same with special keys i suppose
        // TODO: make actions, which would allow for key combos, etc. Maybe relevant for a shell rather, or smth
        if self.key_state.is_pressed() {
            if self.key_state.key == PhysicalKey::Backspace {
                print!("\x08 \x08");
            }
            if self.state_machine.is_key_pressed(PhysicalKey::LeftShift)
                || self.state_machine.is_key_pressed(PhysicalKey::RightShift)
            {
                if let Some(c) = QWERTY_US_LAYOUT_SHIFT[self.key_state.key as usize] {
                    print!("{}", c);
                }
            } else if let Some(c) = QWERTY_US_LAYOUT_NORMAL[self.key_state.key as usize] {
                print!("{}", c);
            }
        }
    }
}

#[derive(Clone, Copy)]
struct KeyState {
    key: PhysicalKey,
    state: KeyStateType,
}

impl KeyState {
    fn from_code(scancode: u8) -> Result<Self, KeyEventError> {
        if scancode == EXTENDED_SCANCODE {
            WAITING_FOR_EXTENDED.store(true, Ordering::SeqCst);
            return Err(KeyEventError::WaitingForExtended(scancode));
        }

        let key_code = scancode & !(1 << 7); // Clear the release bit
        let release_bit = (scancode & (1 << 7)) != 0;
        let is_extended = WAITING_FOR_EXTENDED.load(Ordering::SeqCst);
        let key = if is_extended {
            WAITING_FOR_EXTENDED.store(false, Ordering::SeqCst);
            if key_code == 0x2a || key_code == 0x36 {
                return Err(KeyEventError::Skip(scancode));
            }
            SCAN_CODE_TABLE_EXTENDED[key_code as usize]
        } else {
            SCAN_CODE_TABLE[key_code as usize]
        }
        .ok_or(KeyEventError::UnknownKey(scancode))?;
        let key_state = if release_bit {
            KeyStateType::Released
        } else {
            KeyStateType::Pressed
        };
        Ok(Self {
            key,
            state: key_state,
        })
    }

    const fn is_pressed(self) -> bool {
        matches!(self.state, KeyStateType::Pressed)
    }
}

#[derive(Debug, Clone, Copy)]
enum KeyStateType {
    Released,
    Pressed,
}

#[derive(Debug, Clone, Copy)]
struct KeyStateMachine(u128);

impl KeyStateMachine {
    const fn new() -> Self {
        Self(0)
    }

    const fn set_key_state(&mut self, key: KeyState) -> KeyEvent {
        self.0 = match key.state {
            KeyStateType::Pressed => self.0 | (1 << key.key as u128),
            KeyStateType::Released => self.0 & !(1 << key.key as u128),
        };
        KeyEvent {
            key_state: key,
            state_machine: *self,
        }
    }

    const fn is_key_pressed(&self, key: PhysicalKey) -> bool {
        (self.0 & (1 << key as u128)) != 0
    }
}

#[derive(Debug)]
enum KeyEventError {
    UnknownKey(u8),
    WaitingForExtended(u8),
    Skip(u8),
}

impl Display for KeyEventError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownKey(scancode) => write!(f, "Unknown key scancode: {scancode:#x}"),
            Self::WaitingForExtended(scancode) => {
                write!(f, "Waiting for extended scancode after: {scancode:#x}")
            }
            Self::Skip(scancode) => {
                write!(f, "Skipping scancode: {scancode:#x}")
            }
        }
    }
}

// Keys, add 0x80 to the scan code for key release events

#[expect(clippy::enum_glob_use)]
#[rustfmt::skip]
pub const SCAN_CODE_TABLE: [Option<PhysicalKey>; 128] = {
    use PhysicalKey::*;
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
pub const SCAN_CODE_TABLE_EXTENDED: [Option<PhysicalKey>; 128] = {
    use PhysicalKey::*;
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
#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalKey {
    // Plain (set 1)
    /* 0x01 */ Escape,
    /* 0x02 */ One, Two, Three, Four, Five, Six, Seven, Eight, Nine, Zero, Minus, Equal, Backspace,
    /* 0x0F */ Tab, Q, W, E, R, T, Y, U, I, O, P, LeftBracket, RightBracket, Enter,
    /* 0x1D */ LeftCtrl, A, S, D, F, G, H, J, K, L, Semicolon, Apostrophe, Backtick,
    /* 0x2A */ LeftShift, Backslash, Z, X, C, V, B, N, M, Comma, Period, Slash, RightShift,
    /* 0x37 */ KeypadMultiply, LeftAlt, Space, CapsLock,
    /* 0x3B */ F1, F2, F3, F4, F5, F6, F7, F8, F9, F10,
    /* 0x45 */ NumLock, ScrollLock,
    /* 0x47 */ Keypad7, Keypad8, Keypad9, KeypadMinus,
    /* 0x4B */ Keypad4, Keypad5, Keypad6, KeypadPlus,
    /* 0x4F */ Keypad1, Keypad2, Keypad3, Keypad0, KeypadPeriod,
    /* 0x56 */ NonUsBackslash, F11, F12,

    // Extended (E0)
    /* 0x1C */ KeypadEnter, RightCtrl,
    /* 0x35 */ KeypadDivide, PrintScreen, RightAlt,
    /* 0x47 */ Home, Up, PageUp, Left, Right, End, Down, PageDown, Insert, Delete,
    /* 0x5B */ LeftSuper, RightSuper, Menu,

    // E1 sequence, not in the tables
    // TODO: add that, since it's on my usb keyboard lol
    #[expect(dead_code)]
    Pause,
}

const fn layout(pairs: &[(PhysicalKey, char)]) -> [Option<char>; 128] {
    let mut table = [None; 128];
    let mut i = 0;
    while i < pairs.len() {
        let (key, c) = pairs[i];
        table[key as usize] = Some(c);
        i += 1;
    }
    table
}

#[expect(clippy::enum_glob_use)]
#[rustfmt::skip]
pub const QWERTY_US_LAYOUT_NORMAL: [Option<char>; 128] = {
    use PhysicalKey::*;
    layout(&[
        (Backtick, '`'),
        (One, '1'), (Two, '2'), (Three, '3'), (Four, '4'), (Five, '5'),
        (Six, '6'), (Seven, '7'), (Eight, '8'), (Nine, '9'), (Zero, '0'),
        (Minus, '-'), (Equal, '='),
        (Q, 'q'), (W, 'w'), (E, 'e'), (R, 'r'), (T, 't'),
        (Y, 'y'), (U, 'u'), (I, 'i'), (O, 'o'), (P, 'p'),
        (LeftBracket, '['), (RightBracket, ']'), (Backslash, '\\'),
        (A, 'a'), (S, 's'), (D, 'd'), (F, 'f'), (G, 'g'),
        (H, 'h'), (J, 'j'), (K, 'k'), (L, 'l'),
        (Semicolon, ';'), (Apostrophe, '\''),
        (NonUsBackslash, '\\'),
        (Z, 'z'), (X, 'x'), (C, 'c'), (V, 'v'), (B, 'b'), (N, 'n'), (M, 'm'),
        (Comma, ','), (Period, '.'), (Slash, '/'),
        (Space, ' '), (Enter, '\n'),
        (KeypadDivide, '/'), (KeypadMultiply, '*'), (KeypadMinus, '-'),
        (KeypadPlus, '+'), (KeypadEnter, '\n'),
    ])
};

#[expect(clippy::enum_glob_use)]
#[rustfmt::skip]
pub const QWERTY_US_LAYOUT_SHIFT: [Option<char>; 128] = {
    use PhysicalKey::*;
    layout(&[
        (Backtick, '~'),
        (One, '!'), (Two, '@'), (Three, '#'), (Four, '$'), (Five, '%'),
        (Six, '^'), (Seven, '&'), (Eight, '*'), (Nine, '('), (Zero, ')'),
        (Minus, '_'), (Equal, '+'),
        (Q, 'Q'), (W, 'W'), (E, 'E'), (R, 'R'), (T, 'T'),
        (Y, 'Y'), (U, 'U'), (I, 'I'), (O, 'O'), (P, 'P'),
        (LeftBracket, '{'), (RightBracket, '}'), (Backslash, '|'),
        (A, 'A'), (S, 'S'), (D, 'D'), (F, 'F'), (G, 'G'),
        (H, 'H'), (J, 'J'), (K, 'K'), (L, 'L'),
        (Semicolon, ':'), (Apostrophe, '"'),
        (NonUsBackslash, '|'),
        (Z, 'Z'), (X, 'X'), (C, 'C'), (V, 'V'), (B, 'B'), (N, 'N'), (M, 'M'),
        (Comma, '<'), (Period, '>'), (Slash, '?'),
        (Space, ' '), (Enter, '\n'),
        (KeypadDivide, '/'), (KeypadMultiply, '*'), (KeypadMinus, '-'),
        (KeypadPlus, '+'), (KeypadEnter, '\n'),
    ])
};

/// Only applies when `NumLock` is on
#[expect(clippy::enum_glob_use, dead_code)]
#[rustfmt::skip]
pub const QWERTY_US_LAYOUT_NUMLOCK: [Option<char>; 128] = {
    use PhysicalKey::*;
    layout(&[
        (Keypad7, '7'), (Keypad8, '8'), (Keypad9, '9'),
        (Keypad4, '4'), (Keypad5, '5'), (Keypad6, '6'),
        (Keypad1, '1'), (Keypad2, '2'), (Keypad3, '3'),
        (Keypad0, '0'), (KeypadPeriod, '.'),
    ])
};
