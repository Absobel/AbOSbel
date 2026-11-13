use core::{
    fmt::Display,
    sync::atomic::{AtomicBool, AtomicU128, Ordering},
};

use spin::Mutex;

use crate::{
    arch::{self, idt::InterruptStackFrame, port::Port},
    drivers::{
        keyboard::{
            layout::Layout,
            scancode::{EXTENDED_SCANCODE, PhysicalKey, SCAN_CODE_TABLE, SCAN_CODE_TABLE_EXTENDED},
        },
        pic::{Interrupts, send_eoi},
    },
    print, println,
    ring::RingBuffer,
};

mod layout;
mod scancode;

/// IRQ Keyboard Handler
pub extern "x86-interrupt" fn keyboard(_stack_frame: InterruptStackFrame) {
    let scancode: u8 = unsafe { PS2_DATA_PORT.inb() };
    handle_scancode(scancode);
    send_eoi(Interrupts::Keyboard);
}

// PS/2 Controller

const PS2_DATA_PORT: Port<u8> = Port::new(0x60);
const PS2_COMMAND_PORT: Port<u8> = Port::new(0x64);

pub static WAITING_FOR_EXTENDED: AtomicBool = AtomicBool::new(false);
static KEY_STATE_MACHINE: AtomicU128 = AtomicU128::new(KeyStateMachine::new().0);
pub static KEY_RING_BUFFER: Mutex<RingBuffer<KeyEvent, 16>> = Mutex::new(RingBuffer::new());
// TODO: add something to change that
const KEYBOARD_LAYOUT: Layout = layout::AZERTY_FR_LAYOUT;

pub fn flush_key_events() {
    while let Some(event) = arch::without_interrupts(|| KEY_RING_BUFFER.lock().try_pop()) {
        event.evaluate();
    }

    // To avoid a frozen keyboard because of a full pic buffer
    loop {
        let status = unsafe { PS2_COMMAND_PORT.inb() };
        // output buffer empty
        if status & 0x01 == 0 {
            break;
        }
        let scancode = unsafe { PS2_DATA_PORT.inb() };
        // mouse byte
        if status & 0x20 != 0 {
            continue;
        }
        arch::without_interrupts(|| {
            handle_scancode(scancode);
        });
    }
}

fn handle_scancode(scancode: u8) {
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
                if let Some(c) = KEYBOARD_LAYOUT.shift[self.key_state.key as usize] {
                    print!("{}", c);
                }
            } else if self.state_machine.is_key_pressed(PhysicalKey::RightAlt) {
                if let Some(altgr) = KEYBOARD_LAYOUT.altgr
                    && let Some(c) = altgr[self.key_state.key as usize]
                {
                    print!("{}", c);
                }
            } else if let Some(c) = KEYBOARD_LAYOUT.normal[self.key_state.key as usize] {
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

        if scancode == 0xff {
            WAITING_FOR_EXTENDED.store(false, Ordering::SeqCst);
            return Err(KeyEventError::Skip(scancode));
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
