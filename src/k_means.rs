use std::collections::HashMap;
use std::hash::Hash;

pub trait Data : Eq + Hash {
    fn distance_to(&self, other: &Self) -> u64;
}

pub fn quantize<I>(items: I) -> HashMap<I::Item, I::Item>
    where I: Iterator,
          I::Item: Data {

    let mut count_of_items: HashMap<I::Item, u32> = HashMap::new();

    for item in items {
        let counter = count_of_items.entry(item).or_insert(0);
        *counter += 1;
    }

    count_of_items.shrink_to_fit();

    println!("{:?}", count_of_items.len());

    let mut quantization_map = HashMap::new();
    quantization_map
}
