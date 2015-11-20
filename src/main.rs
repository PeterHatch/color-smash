#![feature(iter_cmp)]

use std::collections::HashMap;
use std::path::Path;

extern crate image;
use image::{DynamicImage, GenericImage};

mod byte_utils;
mod color;
use color::Color;
mod k_means;

#[cfg(test)]
mod tests;

fn main() {
    let mut image = image::open(&Path::new("00.png")).unwrap();

    let quantization_map = quantize_image(&image);

    let mut output_image = image.as_mut_rgba8().unwrap();
    for pixel in output_image.pixels_mut() {
        let &new_color = quantization_map.get(pixel).unwrap();
        *pixel = new_color;
    }

    output_image.save("output.png");
}

fn quantize_image(image: &DynamicImage) -> HashMap<Color, Color> {
    let colors = image.pixels().map(|(_, _, color)| color);
    k_means::quantize(colors)
}
