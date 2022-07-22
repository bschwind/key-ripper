use crate::{key_codes::KeyCode, NUM_COLS, NUM_ROWS};

#[rustfmt::skip]
pub const NORMAL_LAYER_MAPPING: [[KeyCode; NUM_ROWS]; NUM_COLS] = [
    [KeyCode::Escape, KeyCode::Tilde, KeyCode::Tab, KeyCode::CapsLock, KeyCode::LeftShift, KeyCode::Fn],
    [KeyCode::F1, KeyCode::Num1, KeyCode::Q, KeyCode::A, KeyCode::Empty, KeyCode::LeftCtrl],
    [KeyCode::F2, KeyCode::Num2, KeyCode::W, KeyCode::S, KeyCode::Z, KeyCode::LeftAlt],
    [KeyCode::F3, KeyCode::Num3, KeyCode::E, KeyCode::D, KeyCode::X, KeyCode::LeftCmd],
    [KeyCode::F4, KeyCode::Num4, KeyCode::R, KeyCode::F, KeyCode::C, KeyCode::Empty],
    [KeyCode::F5, KeyCode::Num5, KeyCode::T, KeyCode::G, KeyCode::V, KeyCode::Empty],
    [KeyCode::Empty, KeyCode::Num6, KeyCode::Y, KeyCode::H, KeyCode::B, KeyCode::Space],
    [KeyCode::F6, KeyCode::Num7, KeyCode::U, KeyCode::J, KeyCode::N, KeyCode::Empty],
    [KeyCode::F7, KeyCode::Num8, KeyCode::I, KeyCode::K, KeyCode::M, KeyCode::Empty],
    [KeyCode::F8, KeyCode::Num9, KeyCode::O, KeyCode::L, KeyCode::Comma, KeyCode::Empty],
    [KeyCode::F9, KeyCode::Num0, KeyCode::P, KeyCode::Semicolon, KeyCode::Period, KeyCode::RightCmd],
    [KeyCode::F10, KeyCode::Minus, KeyCode::LeftSquareBracket, KeyCode::SingleQuote, KeyCode::ForwardSlash, KeyCode::Left],
    [KeyCode::F11, KeyCode::Equals, KeyCode::RightSquareBracket, KeyCode::Enter, KeyCode::Up, KeyCode::Down],
    [KeyCode::F12, KeyCode::Backspace, KeyCode::BackSlash, KeyCode::Empty, KeyCode::Empty, KeyCode::Right],
];

#[rustfmt::skip]
pub const FN_LAYER_MAPPING: [[KeyCode; NUM_ROWS]; NUM_COLS] = [
    [KeyCode::Escape, KeyCode::Tilde, KeyCode::Tab, KeyCode::CapsLock, KeyCode::LeftShift, KeyCode::Empty],
    [KeyCode::F1, KeyCode::Num1, KeyCode::Q, KeyCode::A, KeyCode::Empty, KeyCode::LeftCtrl],
    [KeyCode::F2, KeyCode::Num2, KeyCode::W, KeyCode::S, KeyCode::Z, KeyCode::LeftAlt],
    [KeyCode::F3, KeyCode::Num3, KeyCode::E, KeyCode::D, KeyCode::X, KeyCode::LeftCmd],
    [KeyCode::F4, KeyCode::Num4, KeyCode::R, KeyCode::F, KeyCode::C, KeyCode::Empty],
    [KeyCode::F5, KeyCode::Num5, KeyCode::T, KeyCode::G, KeyCode::V, KeyCode::Empty],
    [KeyCode::Empty, KeyCode::Num6, KeyCode::Y, KeyCode::H, KeyCode::B, KeyCode::Space],
    [KeyCode::F6, KeyCode::Num7, KeyCode::U, KeyCode::J, KeyCode::N, KeyCode::Empty],
    [KeyCode::F7, KeyCode::Num8, KeyCode::I, KeyCode::K, KeyCode::M, KeyCode::Empty],
    [KeyCode::F8, KeyCode::Num9, KeyCode::O, KeyCode::L, KeyCode::Comma, KeyCode::Empty],
    [KeyCode::F9, KeyCode::Num0, KeyCode::P, KeyCode::Semicolon, KeyCode::Period, KeyCode::RightCmd],
    [KeyCode::VolumeMute, KeyCode::Minus, KeyCode::LeftSquareBracket, KeyCode::SingleQuote, KeyCode::ForwardSlash, KeyCode::Left],
    [KeyCode::VolumeDown, KeyCode::Equals, KeyCode::RightSquareBracket, KeyCode::Enter, KeyCode::Up, KeyCode::Down],
    [KeyCode::VolumeUp, KeyCode::Backspace, KeyCode::BackSlash, KeyCode::Empty, KeyCode::Empty, KeyCode::Right],
];
