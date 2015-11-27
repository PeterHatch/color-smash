extern crate image;
use image::Pixel as PixelTrait;

use byte_utils::*;
use k_means::{SimpleInput, Input, Output, Grouped};

pub type Pixel = image::Rgba<u8>;

pub trait Color {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Self;

    fn as_pixel(self) -> Pixel;

    fn components(&self) -> (u8, u8, u8, u8);
    fn simple_distance_to<T: Color>(&self, other: &T) -> u64 {
        let (r1, g1, b1, a1) = self.components();
        let (r2, g2, b2, a2) = other.components();

        let opaque_distance = ((r1 as i32) - (r2 as i32)).pow(2) +
                              ((g1 as i32) - (g2 as i32)).pow(2) +
                              ((b1 as i32) - (b2 as i32)).pow(2);

        let alpha_distance = ((a1 as i32) - (a2 as i32)).pow(2) * 3;

        ((opaque_distance as u64) * (a1 as u64) * (a2 as u64)) + ((alpha_distance as u64) * 255 * 255)
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Rgba8 {
    pub data: Pixel,
}

impl Color for Rgba8 {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Rgba8 {
        if a > 0 {
            Rgba8 { data: Pixel { data: [r, g, b, a] } }
        } else {
            Rgba8 { data: Pixel { data: [0, 0, 0, 0] } }
        }
    }

    fn as_pixel(self) -> Pixel {
        self.data
    }

    fn components(&self) -> (u8, u8, u8, u8) {
        self.data.channels4()
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Rgb5a3 {
    pub data: Pixel,
}

impl Color for Rgb5a3 {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Rgb5a3 {
        let data = match approximate_3_bits(a) {
            0x00 => [0, 0, 0, 0],
            0xFF => [approximate_5_bits(r), approximate_5_bits(g), approximate_5_bits(b), 0xFF],
            new_a => [approximate_4_bits(r), approximate_4_bits(g), approximate_4_bits(b), new_a],
        };
        Rgb5a3 { data: Pixel { data: data } }
    }

    fn as_pixel(self) -> Pixel {
        self.data
    }

    fn components(&self) -> (u8, u8, u8, u8) {
        self.data.channels4()
    }
}

impl SimpleInput<Rgba8> for Rgba8 {
    fn distance_to(&self, other: &Rgba8) -> u64 {
        self.simple_distance_to(other)
    }

    fn as_output(&self) -> Rgba8 {
        *self
    }
}

impl SimpleInput<Rgb5a3> for Rgba8 {
    fn distance_to(&self, other: &Rgb5a3) -> u64 {
        let closest_possible_distance = self.simple_distance_to::<Rgb5a3>(&self.as_output());
        self.simple_distance_to(other) - closest_possible_distance
    }

    fn as_output(&self) -> Rgb5a3 {
        let (r, g, b, a) = self.components();
        Rgb5a3::new(r, g, b, a)
    }
}

impl Input<Rgba8> for Grouped<Rgba8> {
    fn mean_of(data_and_counts: &Vec<&Grouped<Rgba8>>) -> Rgba8 {
        let mut r_sum = 0;
        let mut g_sum = 0;
        let mut b_sum = 0;
        let mut a_sum = 0;
        let mut total_count = 0;

        for &&Grouped { data: color, count } in data_and_counts {
            let (r, g, b, a) = color.components();
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

            Rgba8::new(r, g, b, a)
        } else {
            Rgba8::new(0, 0, 0, 0)
        }
    }
}

impl Input<Rgb5a3> for Grouped<Rgba8> {
    fn mean_of(data_and_counts: &Vec<&Grouped<Rgba8>>) -> Rgb5a3 {
        let mut r_sum = 0;
        let mut g_sum = 0;
        let mut b_sum = 0;
        let mut a_sum = 0;
        let mut total_count = 0;

        for &&Grouped { data: color, count } in data_and_counts {
            let (r, g, b, a) = color.components();
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

            Rgb5a3::new(r, g, b, a)
        } else {
            Rgb5a3::new(0, 0, 0, 0)
        }
    }
}

impl Output for Rgba8 {}
impl Output for Rgb5a3 {}
