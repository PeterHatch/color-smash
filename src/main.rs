use std::path::Path;

extern crate image;
use image::GenericImage;

mod byte_utils;
mod color;
mod k_means;

#[cfg(test)]
mod tests;

fn main() {
    let image = image::open(&Path::new("00.png")).unwrap();

    let colors = image.pixels().map(|(_, _, color)| color);

    let quantization_map = k_means::quantize(colors);
}
