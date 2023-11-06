use crate::{key_codes::KeyCode, NUM_COLS, NUM_ROWS};

#[rustfmt::skip]
pub const NORMAL_LAYER_MAPPING: [[KeyCode; NUM_ROWS]; NUM_COLS] = [
    [KeyCode::Q, KeyCode::W, KeyCode::E, KeyCode::R, KeyCode::T, KeyCode::Y, KeyCode::U, KeyCode::I, KeyCode::O, KeyCode::P],
    [KeyCode::A, KeyCode::S, KeyCode::D, KeyCode::F, KeyCode::G, KeyCode::H, KeyCode::J, KeyCode::K, KeyCode::L, KeyCode::Semicolon],
    [KeyCode::Z, KeyCode::X, KeyCode::C, KeyCode::V, KeyCode::B, KeyCode::N, KeyCode::M, KeyCode::Comma, KeyCode::Period, KeyCode::ForwardSlash],
    [KeyCode::LeftShift, KeyCode::Fn, KeyCode::Space, KeyCode::Enter, KeyCode::Empty, KeyCode::Empty, KeyCode::Empty, KeyCode::Empty, KeyCode::Empty, KeyCode::Empty],
];

#[rustfmt::skip]
pub const FN_LAYER_MAPPING: [[KeyCode; NUM_ROWS]; NUM_COLS] = [
    [KeyCode::Q, KeyCode::W, KeyCode::E, KeyCode::R, KeyCode::T, KeyCode::Y, KeyCode::U, KeyCode::I, KeyCode::O, KeyCode::P],
    [KeyCode::A, KeyCode::S, KeyCode::D, KeyCode::F, KeyCode::G, KeyCode::H, KeyCode::J, KeyCode::K, KeyCode::L, KeyCode::Semicolon],
    [KeyCode::Z, KeyCode::X, KeyCode::C, KeyCode::V, KeyCode::B, KeyCode::N, KeyCode::M, KeyCode::Comma, KeyCode::Period, KeyCode::ForwardSlash],
    [KeyCode::LeftShift, KeyCode::Empty, KeyCode::Space, KeyCode::Enter, KeyCode::Empty, KeyCode::Empty, KeyCode::Empty, KeyCode::Empty, KeyCode::Empty, KeyCode::Empty],
];
