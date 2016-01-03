use super::quantization_map_from_images;
use color::*;

use std::collections::HashSet;
use std::path::Path;
use image_lib;
use image_lib::RgbaImage;

use test::Bencher;

fn load_test_image() -> RgbaImage {
    image_lib::open(&Path::new("00.png")).unwrap().to_rgba()
}

#[test]
fn has_256_colors() {
    let image = load_test_image();
    let quantization_map = quantization_map_from_images::<Rgb5a3>(&vec![image], false);
    let mut colors = HashSet::new();
    for color in quantization_map.values() {
        colors.insert(color);
    }
    assert_eq!(colors.len(), 256);
}

#[test]
fn rgb_is_zero_if_alpha_is() {
    let image = load_test_image();
    let quantization_map = quantization_map_from_images::<Rgb5a3>(&vec![image], false);
    for colors in quantization_map.values().into_iter().chain(quantization_map.keys().into_iter()) {
        for color in colors {
            if color.data[3] == 0 {
                assert_eq!(color.data, [0, 0, 0, 0]);
            }
        }
    }
}

#[bench]
fn bench_quantization_to_rgb5a3(b: &mut Bencher) {
    let image = load_test_image();
    let images = vec![image];
    b.iter(|| quantization_map_from_images::<Rgb5a3>(&images, false));
}
