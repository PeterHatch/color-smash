//! The main module, which handles the command-line interface.
//!
//! Calls images::quantize to do the actual work of quantization.

#![cfg_attr(test, feature(test))]

use std::env;
use std::path::{Path, PathBuf};

extern crate image as image_lib;
extern crate png;
extern crate num;
extern crate ordered_float;

extern crate getopts;
use getopts::{Matches, Options};

mod color;
mod k_means;
mod images;
mod options;

#[cfg(test)]
extern crate test;

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

    if matches.opt_present("version") {
        print!("color_smash {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let colortype = options::color_type(matches.opt_str("colortype")).unwrap_or_else(|error| {
        println!("{}", error);
        std::process::exit(1);
    });

    if matches.free.is_empty() {
        exit_with_bad_args("No input file specified.", program, options);
    }

    let verbose = matches.opt_present("verbose");

    let input_paths: Vec<&Path> = matches.free
                                         .iter()
                                         .map(|input_string| Path::new(input_string))
                                         .collect();
    let output_pathbufs: Vec<PathBuf> = input_paths.iter()
                                                   .map(|input_path| {
                                                       get_output_path(input_path, &matches)
                                                   })
                                                   .collect();
    let result = images::quantize(input_paths.into_iter(),
                                  output_pathbufs.iter().map(|o| o.as_path()),
                                  colortype,
                                  verbose);

    if let Err(error) = result {
        println!("{}", error);
        std::process::exit(1);
    }
}

fn initialize_options() -> Options {
    let mut options = Options::new();

    options.optflag("h", "help", "print this help message.");
    options.optflag("V", "version", "print version info and exit.");
    options.optflag("v", "verbose", "print detailed output.");
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
