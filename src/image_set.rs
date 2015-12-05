use std::collections::HashMap;
use std::path::Path;

use image_lib;
use image_lib::{GenericImage, RgbaImage, Pixel as PixelTrait, ImageError};

use color_set::ColorSet;
use color::{Color, Pixel, Rgba8, Rgb5a3};

pub fn quantize<'a, 'b, I, O>(input_paths: I, output_paths: O) -> Result<(), ImageError>
    where I: Iterator<Item = &'a Path>,
          O: Iterator<Item = &'b Path> {

    let mut images = Vec::new();
    for input_path in input_paths {
        let image = try!(image_lib::open(input_path));
        images.push(image);
    }
    let mut images: Vec<_> = images.iter_mut().map(|image| image.as_mut_rgba8().unwrap()).collect();

    let quantization_map = create_quantization_map(&images);

    // Temp diagnostic output
    {
        let mut colors = ::std::collections::HashSet::new();
        for color in quantization_map.values() {
            colors.insert(color);
        }
        println!("Colors = {:?}", colors.len());
    }

    let width = images[0].width();
    let height = images[0].height();

    for y in 0..height {
        for x in 0..width {
            let initial_pixels: Vec<_> = images.iter().map(|image| *image.get_pixel(x, y)).collect();
            let new_pixels = quantization_map.get(&initial_pixels).unwrap();
            for (image, &pixel) in images.iter_mut().zip(new_pixels.iter()) {
                image.put_pixel(x, y, pixel);
            }
        }
    }

    for (image, output_path) in images.into_iter().zip(output_paths) {
        try!(image.save(output_path));
    }
    Result::Ok(())
}

pub fn create_quantization_map(images: &Vec<&mut RgbaImage>) -> HashMap<Vec<Pixel>, Vec<Pixel>> {
    let width = images[0].width();
    let height = images[0].height();

    let mut color_sets: Vec<ColorSet<Rgba8>> = Vec::with_capacity((width as usize) * (height as usize));
    for y in 0..height {
        for x in 0..width {
            let color_set = ColorSet::new(images.iter().map(|image| Rgba8::new(image.get_pixel(x, y).channels4())).collect());
            color_sets.push(color_set);
        }
    }

    let grouped_color_sets = ::k_means::collect_groups::<_, ColorSet<Rgb5a3>>(color_sets.into_iter());
    let (centroids, grouped_color_sets_per_centroid): (Vec<ColorSet<Rgb5a3>>, _) = ::k_means::quantize(&grouped_color_sets);

    let mut quantization_map = HashMap::new();

    for (centroid, grouped_color_sets) in centroids.into_iter().zip(grouped_color_sets_per_centroid.iter()) {
        for &grouped_color_set in grouped_color_sets {
            quantization_map.insert(grouped_color_set.clone().data.as_pixels(), centroid.clone().as_pixels());
        }
    }

    quantization_map
}
