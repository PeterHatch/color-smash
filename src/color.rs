extern crate image;
use image::Pixel;

use byte_utils::*;
use k_means::Data;

pub type Color = image::Rgba<u8>;

pub trait ColorUtils {
    fn as_rgb5a3(self) -> Color;
}

impl ColorUtils for Color {
    fn as_rgb5a3(self) -> Color {
        let (r, g, b, a) = self.channels4();
        let new_a = approximate_3_bits(a);
        if new_a == 0xFF {
            Color { data: [approximate_5_bits(r), approximate_5_bits(g), approximate_5_bits(b), new_a] }
        } else {
            Color { data: [approximate_4_bits(r), approximate_4_bits(g), approximate_4_bits(b), new_a] }
        }
    }
}

impl Data for Color {
    fn distance_to(&self, other: &Color) -> u64 {
        let (r1, g1, b1, a1) = self.channels4();
        let (r2, g2, b2, a2) = other.channels4();

        let opaque_distance = ((r1 as i32) - (r2 as i32)).pow(2) +
                              ((g1 as i32) - (g2 as i32)).pow(2) +
                              ((b1 as i32) - (b2 as i32)).pow(2);

        let alpha_distance = ((a1 as i32) - (a2 as i32)).pow(2) * 3;

        ((opaque_distance as u64) * (a1 as u64) * (a2 as u64)) + ((alpha_distance as u64) * 255 * 255)
    }
}
