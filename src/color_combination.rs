use color::{Color, Pixel, Rgba8};
use k_means::{SimpleInput, Input, Output, Grouped};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ColorCombination<T: Color> {
    colors: Vec<T>,
}

impl<T: Color> ColorCombination<T> {
    pub fn new(colors: Vec<T>) -> ColorCombination<T> {
        ColorCombination { colors: colors }
    }
    pub fn as_pixels(self) -> Vec<Pixel> {
        self.colors.into_iter().map(|color| color.as_pixel()).collect()
    }
}

impl<O: Color> SimpleInput<ColorCombination<O>> for ColorCombination<Rgba8> {
    fn distance_to(&self, other: &ColorCombination<O>) -> f64 {
        self.colors
            .iter()
            .zip(other.colors.iter())
            .map(|(c1, c2)| SimpleInput::distance_to(c1, c2))
            .sum()
    }

    fn normalized_distance(&self, other: &ColorCombination<O>) -> f64 {
        self.colors.iter().zip(other.colors.iter()).map(|(c1, c2)| c1.normalized_distance(c2)).sum()
    }

    fn as_output(&self) -> ColorCombination<O> {
        ColorCombination::new(self.colors.iter().map(|color| color.as_output()).collect())
    }
}

impl<O: Color> Input<ColorCombination<O>> for Grouped<ColorCombination<Rgba8>> {
    fn mean_of(grouped_colorsets: &Vec<&Grouped<ColorCombination<Rgba8>>>) -> ColorCombination<O> {
        mean_of(grouped_colorsets)
    }
}

fn mean_of<O: Color>(grouped_colorsets: &Vec<&Grouped<ColorCombination<Rgba8>>>)
                     -> ColorCombination<O> {
    let color_count = grouped_colorsets[0].data.colors.len();
    let mean_colors = (0..color_count)
                          .map(|i| {
                              let color_iter = grouped_colorsets.iter().map(|&group| {
                                  Grouped {
                                      data: group.data.colors[i],
                                      count: group.count,
                                  }
                              });
                              O::new(::color::mean_of_colors(color_iter))
                          })
                          .collect();
    ColorCombination::new(mean_colors)
}

impl<T: Color> Output for ColorCombination<T> {
    fn distance_to(&self, other: &ColorCombination<T>) -> f64 {
        self.colors.iter().zip(other.colors.iter()).map(|(c1, c2)| c1.distance_to(c2)).sum()
    }
}
