pub fn get_bit_at(value: u8, index: u8) -> u8 {
    (value & (1 << index)) >> index
}