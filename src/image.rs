use std::collections::HashMap;
use std::path::Path;

use image_lib;
use image_lib::{GenericImage, RgbaImage, Pixel as PixelTrait, ImageError};

use color::{Color, ColorType, Pixel, Rgba8, Rgb5a3};

use k_means::Output;

pub fn quantize_image(input_file: &Path, output_file: &Path, colortype: ColorType) -> Result<(), ImageError> {
    let mut image = try!(image_lib::open(input_file));
    let mut image = image.as_mut_rgba8().unwrap();

    let quantization_map = create_quantization_map(&image, colortype);

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

    try!(image.save(output_file));
    Result::Ok(())
}

pub fn create_quantization_map(image: &RgbaImage, colortype: ColorType) -> HashMap<Pixel, Pixel> {
    let colors = image.pixels().map(|&color| {
        Rgba8::new(color.channels4())
    });

    match colortype {
        ColorType::Rgba8 => {
            quantize_to::<_, Rgba8>(colors)
        }
        ColorType::Rgb5a3 => {
            quantize_to::<_, Rgb5a3>(colors)
        }
    }
}

fn quantize_to<I, T>(colors: I) -> HashMap<Pixel, Pixel>
    where I: Iterator<Item = Rgba8>,
          T: Color + Output + Copy {
    let grouped_colors = ::k_means::collect_groups::<_, T>(colors);
    let (centroids, grouped_colors_per_centroid): (Vec<T>, _) = ::k_means::quantize(&grouped_colors);

    let mut quantization_map = HashMap::new();

    for (&centroid, grouped_colors) in centroids.iter().zip(grouped_colors_per_centroid.iter()) {
        for &grouped_color in grouped_colors {
            quantization_map.insert(grouped_color.data.as_pixel(), centroid.as_pixel());
        }
    }

    quantization_map
}
