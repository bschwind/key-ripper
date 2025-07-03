use crate::{
    key_codes::KeyCode::{self, *},
    NUM_COLS, NUM_ROWS,
};

#[rustfmt::skip]
pub const NORMAL_LAYER_MAPPING: [[KeyCode; NUM_COLS]; NUM_ROWS] = [
    [Escape, F1, F2, F3, F4, F5, Empty, F6, F7, F8, F9, F10, F11, F12],
    [Tilde, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9, Num0, Minus, Equals, Backspace],
    [Tab, Q, W, E, R, T, Y, U, I, O, P, LeftSquareBracket, RightSquareBracket, BackSlash],
    [LeftCtrl, A, S, D, F, G, H, J, K, L, Semicolon, SingleQuote, Enter, Empty],
    [LeftShift, Empty, Z, X, C, V, B, N, M, Comma, Period, ForwardSlash, Up, Empty],
    [Fn, LeftCtrl, LeftAlt, LeftCmd, Empty, Empty, Space, Empty, Empty, Empty, CapsLock, Left, Down, Right],
];

#[rustfmt::skip]
pub const FN_LAYER_MAPPING: [[KeyCode; NUM_COLS]; NUM_ROWS] = [
    [Escape, F1, F2, F3, F4, F5, Empty, F6, F7, F8, F9, VolumeMute, VolumeDown, VolumeUp],
    [Tilde, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9, Num0, Minus, Equals, Backspace],
    [Tab, Q, W, E, R, T, Y, U, I, O, P, LeftSquareBracket, RightSquareBracket, BackSlash],
    [LeftCtrl, A, S, D, F, G, H, J, K, L, Semicolon, SingleQuote, Enter, Empty],
    [LeftShift, Empty, Z, X, C, V, B, N, M, Comma, Period, ForwardSlash, Up, Empty],
    [Empty, LeftCtrl, LeftAlt, LeftCmd, Empty, Empty, Space, Empty, Empty, Empty, CapsLock, Left, Down, Right],
];
