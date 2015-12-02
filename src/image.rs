use std::collections::HashMap;
use std::path::Path;

use image_lib;
use image_lib::{GenericImage, RgbaImage, Pixel as PixelTrait};

use color::{Color, Pixel, Rgba8, Rgb5a3};

pub fn quantize_image(input_file: &Path, output_file: &Path) {
    let mut image = match image_lib::open(input_file) {
        Ok(image) => { image }
        Err(_) => {
            println!("Could not open {} as an image.", input_file.to_string_lossy());
            ::std::process::exit(1);
        }
    };
    let mut image = image.as_mut_rgba8().unwrap();

    let quantization_map = create_quantization_map(&image);

    // Temp diagnostic output
    {
        let mut colors = ::std::collections::HashSet::new();
        for color in quantization_map.values() {
            colors.insert(color);
        }
        println!("Colors = {:?}", colors.len());
    }

    for pixel in image.pixels_mut() {
        let &new_color = quantization_map.get(pixel).unwrap();
        *pixel = new_color;
    }

    image.save(output_file);
}

pub fn create_quantization_map(image: &RgbaImage) -> HashMap<Pixel, Pixel> {
    let colors = image.pixels().map(|&color| {
        Rgba8::new(color.channels4())
    });
    let grouped_colors = ::k_means::collect_groups::<_, Rgb5a3>(colors);
    let (centroids, grouped_colors_per_centroid): (Vec<Rgb5a3>, _) = ::k_means::quantize(&grouped_colors);

    let mut quantization_map = HashMap::new();

    for (&centroid, grouped_colors) in centroids.iter().zip(grouped_colors_per_centroid.iter()) {
        for &grouped_color in grouped_colors {
            quantization_map.insert(grouped_color.data.as_pixel(), centroid.as_pixel());
        }
    }

    quantization_map
}
