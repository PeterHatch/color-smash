use image_lib;
use image_lib::Pixel as PixelTrait;

use byte_utils::*;
use k_means::{SimpleInput, Input, Output, Grouped};

pub enum ColorType {
    Rgba8,
    Rgb5a3,
}

pub type Pixel = image_lib::Rgba<u8>;

pub trait Color {
    fn new(components: (u8, u8, u8, u8)) -> Self;

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
    fn new(components: (u8, u8, u8, u8)) -> Rgba8 {
        let (r, g, b, a) = components;
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
    fn new(components: (u8, u8, u8, u8)) -> Rgb5a3 {
        let (r, g, b, a) = components;
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

// impl SimpleInput<Rgba8> for Rgba8 {
//     fn distance_to(&self, other: &Rgba8) -> u64 {
//         self.simple_distance_to(other)
//     }

//     fn as_output(&self) -> Rgba8 {
//         *self
//     }
// }

impl<O: Color + Output> SimpleInput<O> for Rgba8 {
    fn distance_to(&self, other: &O) -> u64 {
        let closest_possible_distance = self.simple_distance_to::<O>(&self.as_output());
        let distance = self.simple_distance_to(other);

        if distance < closest_possible_distance {
            println!("Distance from {:?} to {:?} is closer than to RGB5A3 version {:?}", self, other, SimpleInput::<O>::as_output(self));
            return 0;
        }

        distance - closest_possible_distance
    }

    fn as_output(&self) -> O {
        O::new(self.components())
    }
}

impl<O: Color + Output> Input<O> for Grouped<Rgba8> {
    fn mean_of(grouped_colors: &Vec<&Grouped<Rgba8>>) -> O {
        O::new(mean_of_colors_as_vec(grouped_colors))
    }
}

impl Output for Rgba8 {}
impl Output for Rgb5a3 {}

fn mean_of_colors_as_vec(grouped_colors: &Vec<&Grouped<Rgba8>>) -> (u8, u8, u8, u8) {
    mean_of_colors(grouped_colors.iter().map(|&&group| group))
}


pub fn mean_of_colors<I>(grouped_colors: I) -> (u8, u8, u8, u8)
    where I: Iterator<Item = Grouped<Rgba8>> {
    let mut r_sum = 0;
    let mut g_sum = 0;
    let mut b_sum = 0;
    let mut a_sum = 0;
    let mut total_count = 0;

    for Grouped { data, count } in grouped_colors {
        let (r, g, b, a) = data.components();
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

        (r, g, b, a)
    } else {
        (0, 0, 0, 0)
    }
}
