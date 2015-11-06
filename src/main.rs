extern crate image;

use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;

use image::{
    GenericImage,
    Pixel,
    Rgba,
};

fn main() {
    let image = image::open(&Path::new("00.png")).unwrap();

    let colors = image.pixels().map(|(_, _, color)| Color(color));

    let quantization_map: HashMap<Color, Color> = quantize(colors);
}

trait Data : Eq + Hash {

}

impl Data for Color {

}

trait ByteUtils {
    fn convert_8_bits_to_5(self) -> u8;
    fn convert_8_bits_to_4(self) -> u8;
    fn convert_8_bits_to_3(self) -> u8;

    fn convert_5_bits_to_8(self) -> u8;
    fn convert_4_bits_to_8(self) -> u8;
    fn convert_3_bits_to_8(self) -> u8;

    fn approximate_5_bits(self) -> u8;
    fn approximate_4_bits(self) -> u8;
    fn approximate_3_bits(self) -> u8;
}

impl ByteUtils for u8 {
    fn convert_8_bits_to_5(self) -> u8 {
        (((self as u32) * 31 + 127) / 255) as u8
    }
    fn convert_8_bits_to_4(self) -> u8 {
        (((self as u32) + 8) / 17) as u8
    }
    fn convert_8_bits_to_3(self) -> u8 {
        (((self as u32) * 7 + 127) / 255) as u8
    }

    fn convert_5_bits_to_8(self) -> u8 {
        (((self as u32) * 255 + 15) / 31) as u8
    }
    fn convert_4_bits_to_8(self) -> u8 {
        ((self as u32) * 17) as u8
    }
    fn convert_3_bits_to_8(self) -> u8 {
        (((self as u32) * 255 + 3) / 7) as u8
    }

    fn approximate_5_bits(self) -> u8 {
        self.convert_8_bits_to_5().convert_5_bits_to_8()
    }
    fn approximate_4_bits(self) -> u8 {
        self.convert_8_bits_to_4().convert_4_bits_to_8()
    }
    fn approximate_3_bits(self) -> u8 {
        self.convert_8_bits_to_3().convert_3_bits_to_8()
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
struct Color(Rgba<u8>);

impl Color {
    fn as_rgb5a3 (self) -> Color {
        let (r, g, b, a) = self.0.channels4();
        let new_a = a.approximate_3_bits();
        if new_a == 0xFF {
            Color(Rgba { data: [r.approximate_5_bits(), g.approximate_5_bits(), b.approximate_5_bits(), new_a] })
        } else {
            Color(Rgba { data: [r.approximate_4_bits(), g.approximate_4_bits(), b.approximate_4_bits(), new_a] })
        }
    }
}

#[test]
fn color_as_rgb5a3_test() {
    let test_data = [
        ([0xFF, 0x00, 0x08, 0xFF], [0xFF, 0x00, 0x08, 0xFF]),
        ([0xED, 0x04, 0x05, 0xED], [0xEF, 0x00, 0x08, 0xFF]),
        ([0xEC, 0x08, 0x09, 0xEC], [0xEE, 0x00, 0x11, 0xDB]),
    ];
    for &(test_data, expected_data) in &test_data {
        let test_color = Color(Rgba { data: test_data });
        let expected = Color(Rgba { data: expected_data });
        let result = test_color.as_rgb5a3();
        assert_eq!(expected, result);
    }
}

fn quantize<I>(items: I) -> HashMap<I::Item, I::Item>
    where I: Iterator,
          I::Item: Data {

    let mut count_of_items: HashMap<I::Item, u32> = HashMap::new();

    for item in items {
        let counter = count_of_items.entry(item).or_insert(0);
        *counter += 1;
    }

    count_of_items.shrink_to_fit();

    println!("{:?}", count_of_items.len());

    let mut quantization_map = HashMap::new();
    quantization_map
}