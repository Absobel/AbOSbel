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
