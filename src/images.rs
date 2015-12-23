use std::collections::HashMap;
use std::path::Path;

use image_lib;
use image_lib::{GenericImage, RgbaImage, Pixel as PixelTrait, ImageError};

use color_combination::ColorCombination;
use color::{Color, Pixel, Rgba8, Rgb5a3};
use options::ColorType;

pub fn quantize<'a, 'b, I, O>(input_paths: I,
                              output_paths: O,
                              colortype: ColorType)
                              -> Result<(), ImageError>
    where I: Iterator<Item = &'a Path>,
          O: Iterator<Item = &'b Path>
{

    let mut images = Vec::new();
    for input_path in input_paths {
        let image = try!(image_lib::open(input_path));
        images.push(image);
    }
    let mut images: Vec<_> = images.iter_mut().map(|image| image.as_mut_rgba8().unwrap()).collect();

    let quantization_map = create_quantization_map(&images, colortype);

    // Temp diagnostic output
    {
        let mut color_combinations = ::std::collections::HashSet::new();
        for color_combination in quantization_map.values() {
            color_combinations.insert(color_combination);
        }
        println!("{} color combinations in output images",
                 color_combinations.len());
    }

    let width = images[0].width();
    let height = images[0].height();

    for y in 0..height {
        for x in 0..width {
            let initial_pixels: Vec<_> = images.iter()
                                               .map(|image| *image.get_pixel(x, y))
                                               .collect();
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

pub fn create_quantization_map(images: &Vec<&mut RgbaImage>,
                               colortype: ColorType)
                               -> HashMap<Vec<Pixel>, Vec<Pixel>> {
    let color_combinations = get_color_combinations(images);

    match colortype {
        ColorType::Rgba8 => quantize_to::<Rgba8>(color_combinations),
        ColorType::Rgb5a3 => quantize_to::<Rgb5a3>(color_combinations),
    }
}

fn get_color_combinations(images: &Vec<&mut RgbaImage>) -> Vec<ColorCombination<Rgba8>> {
    let width = images[0].width();
    let height = images[0].height();

    let mut color_combinations = Vec::with_capacity((width as usize) * (height as usize));
    for y in 0..height {
        for x in 0..width {
            let color_combination =
                ColorCombination::new(images.iter()
                                            .map(|image| Rgba8::from(*image.get_pixel(x, y)))
                                            .collect());
            color_combinations.push(color_combination);
        }
    }

    color_combinations
}

fn quantize_to<T: Color>(color_combinations: Vec<ColorCombination<Rgba8>>)
                         -> HashMap<Vec<Pixel>, Vec<Pixel>> {
    let grouped_color_combinations =
        ::k_means::collect_groups::<_, ColorCombination<T>>(color_combinations.into_iter());

    println!("{} color combinations in input images",
             grouped_color_combinations.len());

    let (centers, grouped_color_combinations_per_cluster): (Vec<ColorCombination<T>>, _) =
        ::k_means::run(&grouped_color_combinations);

    let mut quantization_map = HashMap::new();

    for (center, grouped_color_combinations) in
        centers.into_iter()
               .zip(grouped_color_combinations_per_cluster.into_iter()) {
        for grouped_color_combination in grouped_color_combinations {
            quantization_map.insert(grouped_color_combination.clone().data.as_pixels(),
                                    center.clone().as_pixels());
        }
    }

    quantization_map
}
