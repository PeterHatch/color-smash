use std::collections::HashMap;
use std::path::Path;

use image_lib;
use image_lib::{DynamicImage, GenericImage, RgbaImage, Pixel as PixelTrait, ImageError};

use color::{Color, Pixel, Rgba8, Rgb5a3};
use color::combination::ConvertibleColorCombination;
use k_means::Grouped;
use options::ColorType;

#[cfg(test)]
mod tests;

pub fn quantize<'a, 'b, I, O>(input_paths: I,
                              output_paths: O,
                              colortype: ColorType)
                              -> Result<(), ImageError>
    where I: Iterator<Item = &'a Path>,
          O: Iterator<Item = &'b Path>
{
    let mut images = try!(open_images(input_paths));
    let mut images = images.iter_mut().map(|image| image.as_mut_rgba8().unwrap()).collect();

    let quantization_map = quantization_map_from_images_and_color_type(&images, colortype);

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
    Ok(())
}

fn open_images<'a, I: Iterator<Item = &'a Path>>(input_paths: I)
                                                 -> Result<Vec<DynamicImage>, ImageError> {
    let mut images = Vec::new();
    for input_path in input_paths {
        let image = try!(image_lib::open(input_path));
        images.push(image);
    }
    Ok(images)
}

fn quantization_map_from_images_and_color_type(images: &Vec<&mut RgbaImage>,
                                               colortype: ColorType)
                                               -> HashMap<Vec<Pixel>, Vec<Pixel>> {
    match colortype {
        ColorType::Rgba8 => quantization_map_from_images::<Rgba8>(images),
        ColorType::Rgb5a3 => quantization_map_from_images::<Rgb5a3>(images),
    }
}

fn quantization_map_from_images<O: Color>(images: &Vec<&mut RgbaImage>)
                                          -> HashMap<Vec<Pixel>, Vec<Pixel>> {
    let color_combinations = get_color_combinations::<O>(images);
    let grouped_color_combinations = group_color_combinations(color_combinations);

    println!("{} color combinations in input images",
             grouped_color_combinations.len());

    quantization_map_from_items(grouped_color_combinations)
}

fn get_color_combinations<O: Color>(images: &Vec<&mut RgbaImage>)
                                    -> Vec<ConvertibleColorCombination<Rgba8, O>> {
    let width = images[0].width();
    let height = images[0].height();

    let mut color_combinations = Vec::with_capacity((width as usize) * (height as usize));
    for y in 0..height {
        for x in 0..width {
            let color_combination =
                ConvertibleColorCombination::<Rgba8, O>::new(images.iter()
                                                                   .map(|image| {
                                                                       (*image.get_pixel(x, y))
                                                                           .into()
                                                                   })
                                                                   .collect());
            color_combinations.push(color_combination);
        }
    }

    color_combinations
}

fn group_color_combinations<O: Color>(color_combinations: Vec<ConvertibleColorCombination<Rgba8, O>>)
                                      -> Vec<Grouped<ConvertibleColorCombination<Rgba8, O>>> {
    ::k_means::collect_groups(color_combinations.into_iter())
}

fn quantization_map_from_items<O: Color>(grouped_color_combinations: Vec<Grouped<ConvertibleColorCombination<Rgba8, O>>>)
                                         -> HashMap<Vec<Pixel>, Vec<Pixel>> {
    let (centers, grouped_color_combinations_per_cluster) =
        ::k_means::run(&grouped_color_combinations);
    let mut quantization_map = HashMap::new();

    for (center, grouped_color_combinations) in centers.into_iter()
                                                       .zip(grouped_color_combinations_per_cluster.into_iter()) {
        for grouped_color_combination in grouped_color_combinations {
            quantization_map.insert(grouped_color_combination.data.as_pixels(),
                                    center.as_pixels());
        }
    }

    quantization_map
}
