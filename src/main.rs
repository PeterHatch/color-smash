#![feature(iter_cmp)]
#![feature(iter_arith)]
#![feature(hashmap_hasher)]

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

extern crate image;
use image::{GenericImage, RgbaImage, Pixel as PixelTrait};

extern crate getopts;
use getopts::{Matches, Options};

mod byte_utils;
mod color;
use color::{Color, Pixel, Rgba8, Rgb5a3};
mod k_means;

#[cfg(test)]
mod tests;

fn main() {
    let mut args = env::args();
    let program = &args.next().unwrap();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help message.");
    opts.optopt("s", "suffix", "set custom suffix for output filenames.", "SUFFIX");

    let matches = match opts.parse(args) {
        Ok(matches) => { matches }
        Err(error) => { exit_with_bad_args(&error.to_string(), program, opts) }
    };

    if matches.opt_present("help") {
        print_usage(program, opts);
        return;
    }
    if matches.free.is_empty() {
        exit_with_bad_args("No input file specified.", program, opts);
    }
    let input_file = Path::new(&matches.free[0]);


    let mut image = match image::open(&input_file) {
        Ok(image) => { image }
        Err(_) => {
            println!("Could not open {} as an image.", input_file.to_string_lossy());
            std::process::exit(1);
        }
    };
    let mut image = image.as_mut_rgba8().unwrap();

    let output_file = output_file(input_file, &matches);

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

    image.save(output_file);
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] FILE", program);
    print!("{}", opts.usage(&brief));
}

fn exit_with_bad_args(error: &str, program: &str, opts: Options) -> ! {
    print!("{}\n\n", error);
    print_usage(program, opts);
    std::process::exit(1);
}

fn output_file(input_file: &Path, matches: &Matches) -> PathBuf {
    let stem = input_file.file_stem().unwrap();
    let output_suffix = match matches.opt_str("suffix") {
        Some(suffix) => { suffix }
        None => { " (crushed)".to_string() }
    };
    let output_extension = ".png";
    let output_name = stem.to_string_lossy().into_owned() + &output_suffix + output_extension;
    input_file.with_file_name(output_name)
}

fn quantize_image(image: &RgbaImage) -> HashMap<Pixel, Pixel> {
    let colors = image.pixels().map(|&color| {
        Rgba8::new(color.channels4())
    });
    let grouped_colors = k_means::collect_groups::<_, Rgb5a3>(colors);
    let (centroids, grouped_colors_per_centroid): (Vec<Rgb5a3>, _) = k_means::quantize(&grouped_colors);

    let mut quantization_map = HashMap::new();

    for (&centroid, grouped_colors) in centroids.iter().zip(grouped_colors_per_centroid.iter()) {
        for &grouped_color in grouped_colors {
            quantization_map.insert(grouped_color.data.as_pixel(), centroid.as_pixel());
        }
    }

    quantization_map
}
