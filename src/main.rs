#![feature(iter_cmp)]
#![feature(iter_arith)]

use std::collections::HashMap;
use std::path::Path;

extern crate image;
use image::{GenericImage, RgbaImage};

mod byte_utils;
mod color;
use color::Color;
mod k_means;

#[cfg(test)]
mod tests;

fn main() {
    let mut image = image::open(&Path::new("00.png")).unwrap();
    let mut image = image.as_mut_rgba8().unwrap();

    let quantization_map = quantize_image(&image);

    // Temp diagnostic output
    {
        let mut colors = std::collections::HashSet::new();
        for color in quantization_map.values() {
            colors.insert(color);
        }
        println!("Colors = {:?}", colors.len());
    }

    for pixel in image.pixels_mut() {
        let &new_color = quantization_map.get(pixel).unwrap();
        *pixel = new_color;
    }

    image.save("output.png");
}

fn quantize_image(image: &RgbaImage) -> HashMap<Color, Color> {
    let colors = image.pixels().map(|&color| {
        if color.data[3] == 0 {
            Color { data: [0, 0, 0, 0] }
        } else {
            color
        }
    });
    let grouped_colors = k_means::collect_groups(colors);
    let (centroids, grouped_colors_per_centroid) = k_means::quantize(&grouped_colors);

    let mut quantization_map = HashMap::new();

    for (&centroid, grouped_colors) in centroids.iter().zip(grouped_colors_per_centroid.iter()) {
        for &grouped_color in grouped_colors {
            quantization_map.insert(grouped_color.data, centroid);
        }
    }

    quantization_map
}
