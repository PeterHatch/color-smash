use color::{Color, ColorUtils};
use k_means::Data;

#[test]
fn color_as_rgb5a3_test() {
    let test_data = [
        ([0xFF, 0x00, 0x08, 0xFF], [0xFF, 0x00, 0x08, 0xFF]),
        ([0xED, 0x04, 0x05, 0xED], [0xEF, 0x00, 0x08, 0xFF]),
        ([0xEC, 0x08, 0x09, 0xEC], [0xEE, 0x00, 0x11, 0xDB]),
    ];
    for &(test_data, expected_data) in &test_data {
        let test_color = Color { data: test_data };
        let expected = Color { data: expected_data };
        let result = test_color.as_rgb5a3();
        assert_eq!(expected, result);
    }
}

#[test]
fn color_distance_test() {
    let test_data = [
        ([0xFF, 0xFF, 0xFF, 0xFF], [0x00, 0x00, 0x00, 0xFF], 12_684_751_875),
        ([0xFF, 0xFF, 0xFF, 0xFF], [0x00, 0x00, 0x00, 0x00], 12_684_751_875),
    ];
    for &(first_color, second_color, expected_distance) in &test_data {
        let first = Color { data: first_color };
        let second = Color { data: second_color };
        let result = first.distance_to(&second);
        assert_eq!(expected_distance, result);
    }
}

#[test]
fn color_mean_test() {
    let test_data = [
        ([([0xFF, 0x80, 0x00, 0xFF], 1), ([0x00, 0x00, 0x00, 0xFF], 1)], [0x80, 0x40, 0x00, 0xFF]),
        ([([0xFF, 0xFF, 0xFF, 0x00], 1), ([0x80, 0x80, 0x80, 0x00], 1)], [0x00, 0x00, 0x00, 0x00]),
        ([([0xFF, 0x80, 0x00, 0x80], 2), ([0x00, 0x00, 0x00, 0xFF], 1)], [0x80, 0x40, 0x00, 0xAA]),
    ];
    for &(colors, expected_data) in &test_data {
        let vector: Vec<_> = colors.iter().map(|&(color_data, count)| { (Color { data: color_data }, count) }).collect();
        let expected_mean = Color { data: expected_data };
        let result = Color::mean_of(&vector);
        assert_eq!(expected_mean, result);
    }
}
