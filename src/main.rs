#![feature(iter_arith)]
#![feature(hashmap_hasher)]

use std::env;
use std::ops::Deref;
use std::path::{Path, PathBuf};

extern crate image as image_lib;

extern crate getopts;
use getopts::{Matches, Options};

mod byte_utils;
mod color;
use color::ColorType;
mod color_set;
mod k_means;
mod image_set;
mod numeric_float;

#[cfg(test)]
mod tests;

fn main() {
    let mut args = env::args();
    let program = &args.next().unwrap();

    let options = initialize_options();

    let matches = match options.parse(args) {
        Ok(matches) => matches,
        Err(error) => exit_with_bad_args(&error.to_string(), program, options),
    };

    if matches.opt_present("help") {
        print_usage(program, options);
        return;
    }

    let colortype = match matches.opt_str("colortype") {
        Some(string) => {
            let colortype = string.to_uppercase();
            match colortype.deref() {
                "RGBA8" => ColorType::Rgba8,
                "RGB5A3" => ColorType::Rgb5a3,
                _ => {
                    println!("Unknown color type {}", string);
                    std::process::exit(1);
                }
            }
        }
        None => ColorType::Rgba8,
    };

    if matches.free.is_empty() {
        exit_with_bad_args("No input file specified.", program, options);
    }

    let input_paths: Vec<&Path> = matches.free
                                         .iter()
                                         .map(|input_string| Path::new(input_string))
                                         .collect();
    let output_pathbufs: Vec<PathBuf> = input_paths.iter()
                                                   .map(|input_path| {
                                                       get_output_path(input_path, &matches)
                                                   })
                                                   .collect();
    let result = image_set::quantize(input_paths.into_iter(),
                                     output_pathbufs.iter().map(|o| o.as_path()),
                                     colortype);

    if let Err(error) = result {
        println!("{}", error);
        std::process::exit(1);
    }
}

fn initialize_options() -> Options {
    let mut options = Options::new();

    options.optflag("h", "help", "print this help message.");
    options.optopt("s",
                   "suffix",
                   "set custom suffix for output filenames.",
                   "SUFFIX");
    options.optopt("c",
                   "colortype",
                   "set output to RGBA8 (default) or RGB5A3.",
                   "TYPE");

    options
}

fn print_usage(program: &str, options: Options) {
    let brief = format!("Usage: {} [options] FILE", program);
    print!("{}", options.usage(&brief));
}

fn exit_with_bad_args(error: &str, program: &str, options: Options) -> ! {
    print!("{}\n\n", error);
    print_usage(program, options);
    std::process::exit(1);
}

fn get_output_path(input_file: &Path, matches: &Matches) -> PathBuf {
    let stem = input_file.file_stem().unwrap();
    let output_suffix = match matches.opt_str("suffix") {
        Some(suffix) => suffix,
        None => " (smashed)".to_string(),
    };
    let output_extension = ".png";
    let output_name = stem.to_string_lossy().into_owned() + &output_suffix + output_extension;
    input_file.with_file_name(output_name)
}
