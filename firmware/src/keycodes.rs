#[repr(u8)]
pub enum KeyCode {
    A = 0x04,
    B = 0x05,
    C = 0x06,
    D = 0x07,
    Escape = 0x29,
    Tab = 0x2B,
    Tilde = 0x35,
    CapsLock = 0x39,
    Fn = 0x98,        // TODO - add as modifier
    LeftShift = 0x99, // TODO - add as modifier
}
