pub fn get_bit_at(value: u8, index: u8) -> u8 {
    (value & (1 << index)) >> index
}

pub const CARRY: u8 = 0;
pub const ZERO: u8 = 1;
pub const INTERRUPT: u8 = 2;
pub const DECIMAL: u8 = 3;
pub const OVERFLOW: u8 = 6;
pub const NEGATIVE: u8 = 7;

pub const SET: u8 = 1;
pub const CLEAR: u8 = 0;