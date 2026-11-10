use crate::drivers::keyboard::scancode::PhysicalKey;

#[expect(dead_code)]
pub struct Layout {
    pub normal: [Option<char>; 128],
    pub shift: [Option<char>; 128],
    numlock: [Option<char>; 128],
    pub altgr: Option<[Option<char>; 128]>,
}

#[expect(dead_code)]
pub const QWERTY_US_LAYOUT: Layout = Layout {
    normal: QWERTY_US_LAYOUT_NORMAL,
    shift: QWERTY_US_LAYOUT_SHIFT,
    numlock: QWERTY_US_LAYOUT_NUMLOCK,
    altgr: None,
};

pub const AZERTY_FR_LAYOUT: Layout = Layout {
    normal: AZERTY_FR_LAYOUT_NORMAL,
    shift: AZERTY_FR_LAYOUT_SHIFT,
    numlock: QWERTY_US_LAYOUT_NUMLOCK,
    altgr: Some(AZERTY_FR_LAYOUT_ALTGR),
};

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
#[expect(clippy::enum_glob_use)]
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

// TODO: add the missing keys when ^ and ¨ are supported as dead keys
#[expect(clippy::enum_glob_use)]
#[rustfmt::skip]
pub const AZERTY_FR_LAYOUT_NORMAL: [Option<char>; 128] = {
    use PhysicalKey::*;
    layout(&[
        (Backtick, '²'),
        (One, '&'), (Two, 'é'), (Three, '"'), (Four, '\''), (Five, '('),
        (Six, '-'), (Seven, 'è'), (Eight, '_'), (Nine, 'ç'), (Zero, 'à'),
        (Minus, ')'), (Equal, '='),
        (Q, 'a'), (W, 'z'), (E, 'e'), (R, 'r'), (T, 't'),
        (Y, 'y'), (U, 'u'), (I, 'i'), (O, 'o'), (P, 'p'),
        (RightBracket, '$'),
        (A, 'q'), (S, 's'), (D, 'd'), (F, 'f'), (G, 'g'),
        (H, 'h'), (J, 'j'), (K, 'k'), (L, 'l'),
        (Semicolon, 'm'), (Apostrophe, 'ù'), (Backslash, '*'),
        (NonUsBackslash, '<'),
        (Z, 'w'), (X, 'x'), (C, 'c'), (V, 'v'), (B, 'b'), (N, 'n'),
        (M, ','), (Comma, ';'), (Period, ':'), (Slash, '!'),
        (Space, ' '), (Enter, '\n'),
        (KeypadDivide, '/'), (KeypadMultiply, '*'), (KeypadMinus, '-'),
        (KeypadPlus, '+'), (KeypadEnter, '\n'),
    ])
};

#[expect(clippy::enum_glob_use)]
#[rustfmt::skip]
pub const AZERTY_FR_LAYOUT_SHIFT: [Option<char>; 128] = {
    use PhysicalKey::*;
    layout(&[
        (One, '1'), (Two, '2'), (Three, '3'), (Four, '4'), (Five, '5'),
        (Six, '6'), (Seven, '7'), (Eight, '8'), (Nine, '9'), (Zero, '0'),
        (Minus, '°'), (Equal, '+'),
        (Q, 'A'), (W, 'Z'), (E, 'E'), (R, 'R'), (T, 'T'),
        (Y, 'Y'), (U, 'U'), (I, 'I'), (O, 'O'), (P, 'P'),
        (RightBracket, '£'),
        (A, 'Q'), (S, 'S'), (D, 'D'), (F, 'F'), (G, 'G'),
        (H, 'H'), (J, 'J'), (K, 'K'), (L, 'L'),
        (Semicolon, 'M'), (Apostrophe, '%'), (Backslash, 'µ'),
        (NonUsBackslash, '>'),
        (Z, 'W'), (X, 'X'), (C, 'C'), (V, 'V'), (B, 'B'), (N, 'N'),
        (M, '?'), (Comma, '.'), (Period, '/'), (Slash, '§'),
        (Space, ' '), (Enter, '\n'),
        (KeypadDivide, '/'), (KeypadMultiply, '*'), (KeypadMinus, '-'),
        (KeypadPlus, '+'), (KeypadEnter, '\n'),
    ])
};

#[expect(clippy::enum_glob_use)]
#[rustfmt::skip]
pub const AZERTY_FR_LAYOUT_ALTGR: [Option<char>; 128] = {
    use PhysicalKey::*;
    layout(&[
        (Two, '~'), (Three, '#'), (Four, '{'), (Five, '['), (Six, '|'),
        (Seven, '`'), (Eight, '\\'), (Nine, '^'), (Zero, '@'),
        (Minus, ']'), (Equal, '}'),
    ])
};
