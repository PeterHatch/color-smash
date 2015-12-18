pub fn convert_5_bits_to_8(byte: u16) -> u8 {
    ((byte * 255 + 15) / 31) as u8
}
pub fn convert_4_bits_to_8(byte: u16) -> u8 {
    (byte * 17) as u8
}
pub fn convert_3_bits_to_8(byte: u16) -> u8 {
    ((byte * 255 + 3) / 7) as u8
}
