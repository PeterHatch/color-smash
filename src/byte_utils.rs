fn convert_8_bits_to_5(byte: u8) -> u8 {
    (((byte as u32) * 31 + 127) / 255) as u8
}
fn convert_8_bits_to_4(byte: u8) -> u8 {
    (((byte as u32) + 8) / 17) as u8
}
fn convert_8_bits_to_3(byte: u8) -> u8 {
    (((byte as u32) * 7 + 127) / 255) as u8
}

fn convert_5_bits_to_8(byte: u8) -> u8 {
    (((byte as u32) * 255 + 15) / 31) as u8
}
fn convert_4_bits_to_8(byte: u8) -> u8 {
    ((byte as u32) * 17) as u8
}
fn convert_3_bits_to_8(byte: u8) -> u8 {
    (((byte as u32) * 255 + 3) / 7) as u8
}

pub fn approximate_5_bits(byte: u8) -> u8 {
    convert_5_bits_to_8(convert_8_bits_to_5(byte))
}
pub fn approximate_4_bits(byte: u8) -> u8 {
    convert_4_bits_to_8(convert_8_bits_to_4(byte))
}
pub fn approximate_3_bits(byte: u8) -> u8 {
    convert_3_bits_to_8(convert_8_bits_to_3(byte))
}
