extern crate image;
use image::Pixel;

use byte_utils::*;
use k_means::{Data, Node};

pub type Color = image::Rgba<u8>;

pub trait ColorUtils {
    fn as_rgb5a3(self) -> Color;
    fn simple_distance_to(&self, other: &Color) -> u64;
}

impl ColorUtils for Color {
    fn as_rgb5a3(self) -> Color {
        let (r, g, b, a) = self.channels4();
        match approximate_3_bits(a) {
            0x00 => Color { data: [0, 0, 0, 0] },
            0xFF => Color { data: [approximate_5_bits(r), approximate_5_bits(g), approximate_5_bits(b), 0xFF] },
            new_a => Color { data: [approximate_4_bits(r), approximate_4_bits(g), approximate_4_bits(b), new_a] },
        }
    }

    fn simple_distance_to(&self, other: &Color) -> u64 {
        let (r1, g1, b1, a1) = self.channels4();
        let (r2, g2, b2, a2) = other.channels4();

        let opaque_distance = ((r1 as i32) - (r2 as i32)).pow(2) +
                              ((g1 as i32) - (g2 as i32)).pow(2) +
                              ((b1 as i32) - (b2 as i32)).pow(2);

        let alpha_distance = ((a1 as i32) - (a2 as i32)).pow(2) * 3;

        ((opaque_distance as u64) * (a1 as u64) * (a2 as u64)) + ((alpha_distance as u64) * 255 * 255)
    }
}

impl Data for Color {
    type Output = Color;

    fn distance_to(&self, other: &Color) -> u64 {
        let closest_possible_distance = self.simple_distance_to(&self.as_output());
        self.simple_distance_to(other) - closest_possible_distance
    }

    fn mean_of(data_and_counts: &Vec<&Node<Color>>) -> Color {
        let mut r_sum = 0;
        let mut g_sum = 0;
        let mut b_sum = 0;
        let mut a_sum = 0;
        let mut total_count = 0;

        for &&Node { data: color, count } in data_and_counts {
            let (r, g, b, a) = color.channels4();
            let weighted_a = (a as u32) * count;

            r_sum += (r as u32) * weighted_a;
            g_sum += (g as u32) * weighted_a;
            b_sum += (b as u32) * weighted_a;
            a_sum += weighted_a;
            total_count += count;
        }

        if a_sum > 0 {
            let r = ((r_sum + (a_sum / 2)) / a_sum) as u8;
            let g = ((g_sum + (a_sum / 2)) / a_sum) as u8;
            let b = ((b_sum + (a_sum / 2)) / a_sum) as u8;
            let a = ((a_sum + (total_count / 2)) / total_count) as u8;

            Color { data: [r, g, b, a] }.as_output()
        } else {
            Color { data: [0, 0, 0, 0] }
        }
    }

    fn as_output(&self) -> Color {
        self.as_rgb5a3()
    }
}
