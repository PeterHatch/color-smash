use std::fmt;

use image_lib;
use image_lib::Pixel as PixelTrait;

use k_means::{SimpleInput, Input, Output, Grouped};

#[cfg(test)]
mod tests;

pub enum ColorType {
    Rgba8,
    Rgb5a3,
}

pub type Pixel = image_lib::Rgba<u8>;

pub trait Color {
    fn new(components: (f64, f64, f64, f64)) -> Self;

    fn as_pixel(self) -> Pixel;

    fn components(&self) -> (f64, f64, f64, f64);
    fn simple_distance_to<T: Color>(&self, other: &T) -> f64 {
        let (r1, g1, b1, a1) = self.components();
        let (r2, g2, b2, a2) = other.components();

        let opaque_distance = (r1 - r2).powi(2) + (g1 - g2).powi(2) + (b1 - b2).powi(2);
        let alpha_distance = (a1 - a2).powi(2) * 3.0;

        (opaque_distance * a1 * a2) + alpha_distance
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Rgba8 {
    data: Pixel,
}

impl Color for Rgba8 {
    fn new(components: (f64, f64, f64, f64)) -> Rgba8 {
        let (r_float, g_float, b_float, a_float) = components;

        let a = (a_float * 255.0).round() as u8;
        if a == 0 {
            return Rgba8 { data: Pixel { data: [0, 0, 0, 0] } };
        }

        let r = (r_float * 255.0).round() as u8;
        let g = (g_float * 255.0).round() as u8;
        let b = (b_float * 255.0).round() as u8;
        Rgba8 { data: Pixel { data: [r, g, b, a] } }
    }

    fn as_pixel(self) -> Pixel {
        self.data
    }

    fn components(&self) -> (f64, f64, f64, f64) {
        let (r, g, b, a) = self.data.channels4();
        ((r as f64) / 255.0,
         (g as f64) / 255.0,
         (b as f64) / 255.0,
         (a as f64) / 255.0)
    }
}

impl From<Pixel> for Rgba8 {
    fn from(pixel: Pixel) -> Self {
        Rgba8 { data: pixel }
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct Rgb5a3 {
    data: u16,
}

enum Rgb5a3Type {
    Rgb5,
    Rgb4a3,
}

impl Rgb5a3 {
    fn storage_type(&self) -> Rgb5a3Type {
        if (self.data >> 15) & 1 == 1 {
            Rgb5a3Type::Rgb5
        } else {
            Rgb5a3Type::Rgb4a3
        }
    }

    fn r5(&self) -> u16 {
        (self.data >> 10) & 0x1F
    }
    fn g5(&self) -> u16 {
        (self.data >> 5) & 0x1F
    }
    fn b5(&self) -> u16 {
        self.data & 0x1F
    }

    fn a3(&self) -> u16 {
        (self.data >> 12) & 0x07
    }
    fn r4(&self) -> u16 {
        (self.data >> 8) & 0x0F
    }
    fn g4(&self) -> u16 {
        (self.data >> 4) & 0x0F
    }
    fn b4(&self) -> u16 {
        self.data & 0x0F
    }
}

fn convert_5_bits_to_8(byte: u16) -> u8 {
    ((byte * 255 + 15) / 31) as u8
}
fn convert_4_bits_to_8(byte: u16) -> u8 {
    (byte * 17) as u8
}
fn convert_3_bits_to_8(byte: u16) -> u8 {
    ((byte * 255 + 3) / 7) as u8
}

impl Color for Rgb5a3 {
    fn new(components: (f64, f64, f64, f64)) -> Rgb5a3 {
        let (r_float, g_float, b_float, a_float) = components;
        let a = (a_float * 7.0).round() as u16;

        let data = match a {
            0 => 0,
            7 => {
                let r = (r_float * 31.0).round() as u16;
                let g = (g_float * 31.0).round() as u16;
                let b = (b_float * 31.0).round() as u16;
                (1 << 15) | (r << 10) | (g << 5) | b
            }
            1...7 => {
                let r = (r_float * 15.0).round() as u16;
                let g = (g_float * 15.0).round() as u16;
                let b = (b_float * 15.0).round() as u16;
                (a << 12) | (r << 8) | (g << 4) | b
            }
            _ => {
                panic!("Invalid alpha parameter to Rgb5a3::new: {:?} (as 3 bit integer {:?})",
                       a_float,
                       a)
            }
        };
        Rgb5a3 { data: data }
    }

    fn as_pixel(self) -> Pixel {
        match self.storage_type() {
            Rgb5a3Type::Rgb5 => {
                let r = convert_5_bits_to_8(self.r5());
                let g = convert_5_bits_to_8(self.g5());
                let b = convert_5_bits_to_8(self.b5());
                Pixel { data: [r, g, b, 0xFF] }
            }
            Rgb5a3Type::Rgb4a3 => {
                let a = convert_3_bits_to_8(self.a3());
                let r = convert_4_bits_to_8(self.r4());
                let g = convert_4_bits_to_8(self.g4());
                let b = convert_4_bits_to_8(self.b4());
                Pixel { data: [r, g, b, a] }
            }
        }
    }

    fn components(&self) -> (f64, f64, f64, f64) {
        match self.storage_type() {
            Rgb5a3Type::Rgb5 => {
                let r = (self.r5() as f64) / 31.0;
                let g = (self.g5() as f64) / 31.0;
                let b = (self.b5() as f64) / 31.0;
                (r, g, b, 1.0)
            }
            Rgb5a3Type::Rgb4a3 => {
                let a = (self.a3() as f64) / 7.0;
                let r = (self.r4() as f64) / 15.0;
                let g = (self.g4() as f64) / 15.0;
                let b = (self.b4() as f64) / 15.0;
                (r, g, b, a)
            }
        }
    }
}

impl fmt::Debug for Rgb5a3 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.storage_type() {
            Rgb5a3Type::Rgb5 => {
                fmt.debug_struct("Rgb5a3 (Rgb5)")
                   .field("r", &self.r5())
                   .field("g", &self.g5())
                   .field("b", &self.b5())
                   .finish()
            }
            Rgb5a3Type::Rgb4a3 => {
                fmt.debug_struct("Rgb5a3 (Rgb4a3)")
                   .field("r", &self.r4())
                   .field("g", &self.g4())
                   .field("b", &self.b4())
                   .field("a", &self.a3())
                   .finish()
            }
        }
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
    fn distance_to(&self, other: &O) -> f64 {
        let closest_possible_distance = self.simple_distance_to::<O>(&self.as_output());
        let distance = self.simple_distance_to(other);

        if distance < closest_possible_distance {
            println!("Distance from {:?} to {:?} is closer than to output version {:?}",
                     self,
                     other,
                     SimpleInput::<O>::as_output(self));
            return 0.0;
        }

        distance - closest_possible_distance
    }

    fn as_output(&self) -> O {
        O::new(self.components())
    }
}

impl<O: Color + Output> Input<O> for Grouped<Rgba8> {
    fn mean_of(grouped_colors: &Vec<&Grouped<Rgba8>>) -> O {
        O::new(mean_of_colors(grouped_colors.iter().map(|&&group| group)))
    }
}

pub fn mean_of_colors<I>(grouped_colors: I) -> (f64, f64, f64, f64)
    where I: Iterator<Item = Grouped<Rgba8>>
{
    let mut r_sum = 0.0;
    let mut g_sum = 0.0;
    let mut b_sum = 0.0;
    let mut a_sum = 0.0;
    let mut total_count = 0;

    for Grouped { data, count } in grouped_colors {
        let (r, g, b, a) = data.components();
        let weighted_a = a * (count as f64);

        r_sum += r * weighted_a;
        g_sum += g * weighted_a;
        b_sum += b * weighted_a;
        a_sum += weighted_a;
        total_count += count;
    }

    if a_sum > 0.0 {
        let r = r_sum / a_sum;
        let g = g_sum / a_sum;
        let b = b_sum / a_sum;
        let a = a_sum / (total_count as f64);

        (r, g, b, a)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    }
}

impl Output for Rgba8 {}
impl Output for Rgb5a3 {}
