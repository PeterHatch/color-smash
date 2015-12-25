use color::{Color, InputColor, Pixel};
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

impl<T: Color> Output for ColorCombination<T> {
    fn distance_to(&self, other: &ColorCombination<T>) -> f64 {
        self.colors.iter().zip(other.colors.iter()).map(|(c1, c2)| c1.distance_to(c2)).sum()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct InputColorCombination<I: Color, O: Color> {
    colors: Vec<InputColor<I, O>>,
}

impl<I: Color, O: Color> InputColorCombination<I, O> {
    pub fn new(colors: Vec<InputColor<I, O>>) -> InputColorCombination<I, O> {
        InputColorCombination { colors: colors }
    }
    pub fn as_pixels(self) -> Vec<Pixel> {
        self.colors.into_iter().map(|input_color| input_color.color.as_pixel()).collect()
    }
}

impl<I: Color, O: Color> SimpleInput for InputColorCombination<I, O> {
    type Output = ColorCombination<O>;

    fn distance_to(&self, other: &Self::Output) -> f64 {
        self.colors
            .iter()
            .zip(other.colors.iter())
            .map(|(c1, c2)| c1.distance_to(c2))
            .sum()
    }

    fn normalized_distance(&self, other: &Self::Output) -> f64 {
        self.colors
            .iter()
            .zip(other.colors.iter())
            .map(|(c1, c2)| c1.normalized_distance(c2))
            .sum()
    }

    fn as_output(&self) -> Self::Output {
        ColorCombination::new(self.colors.iter().map(|color| color.as_output()).collect())
    }
}

impl<I: Color, O: Color> Input for Grouped<InputColorCombination<I, O>> {
    fn mean_of(grouped_colorsets: &Vec<&Grouped<InputColorCombination<I, O>>>) -> Self::Output {
        mean_of(grouped_colorsets)
    }
}

fn mean_of<I: Color, O: Color>(grouped_colorsets: &Vec<&Grouped<InputColorCombination<I, O>>>)
                               -> ColorCombination<O> {
    let color_count = grouped_colorsets[0].data.colors.len();
    let mean_colors = (0..color_count)
                          .map(|i| {
                              let color_iter = grouped_colorsets.iter().map(|&group| {
                                  (&group.data.colors[i], group.count)
                              });
                              ::color::mean_of_colors(color_iter)
                          })
                          .collect();
    ColorCombination::new(mean_colors)
}
