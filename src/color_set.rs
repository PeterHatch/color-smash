use color::{Color, Pixel, Rgba8};
use k_means::{SimpleInput, Input, Output, Grouped};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ColorSet<T: Color> {
    colors: Vec<T>,
}

impl<T: Color> ColorSet<T> {
    pub fn new(colors: Vec<T>) -> ColorSet<T> {
        ColorSet { colors: colors }
    }
    pub fn as_pixels(self) -> Vec<Pixel> {
        self.colors.into_iter().map(|color| color.as_pixel()).collect()
    }
}

impl<O: Color + Output> SimpleInput<ColorSet<O>> for ColorSet<Rgba8> {
    fn distance_to(&self, other: &ColorSet<O>) -> f64 {
        self.colors.iter().zip(other.colors.iter()).map(|(c1, c2)| c1.distance_to(c2)).sum()
    }

    fn as_output(&self) -> ColorSet<O> {
        ColorSet::new(self.colors.iter().map(|color| color.as_output()).collect())
    }
}

impl<O: Color + Output> Input<ColorSet<O>> for Grouped<ColorSet<Rgba8>> {
    fn mean_of(grouped_colorsets: &Vec<&Grouped<ColorSet<Rgba8>>>) -> ColorSet<O> {
        mean_of(grouped_colorsets)
    }
}

fn mean_of<O: Color + Output>(grouped_colorsets: &Vec<&Grouped<ColorSet<Rgba8>>>) -> ColorSet<O> {
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
    ColorSet::new(mean_colors)
}

impl<T: Color + Output> Output for ColorSet<T> {}
