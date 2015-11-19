use std::collections::HashMap;
use std::hash::Hash;

pub trait Data : Eq + Hash + Copy + Clone {
    fn distance_to(&self, other: &Self) -> u64;
    fn mean_of(data_and_counts: &Vec<Node<Self>>) -> Self;
}

pub struct Node<T: Data> {
    pub data: T,
    pub count: u32,
}

impl<T: Data> Node<T> {
    fn to_centroid(&self) -> Centroid<T> {
        Centroid { data: self.data }
    }
}

struct Centroid<T: Data> {
    data: T,
}

pub fn quantize<I>(items: I) -> HashMap<I::Item, I::Item>
    where I: Iterator,
          I::Item: Data {
    let k = 256;

    let nodes = create_nodes(items);
    let mut centroids = initialize_centroids(k, &nodes);

    println!("{:?}", nodes.len());
    println!("{:?}", centroids.len());

    let mut quantization_map = HashMap::new();
    quantization_map
}

fn create_nodes<I>(items: I) -> Vec<Node<I::Item>>
    where I: Iterator,
          I::Item: Data {

    let mut count_of_items: HashMap<I::Item, u32> = HashMap::new();
    for item in items {
        let counter = count_of_items.entry(item).or_insert(0);
        *counter += 1;
    }

    let mut nodes = Vec::with_capacity(count_of_items.len());
    for (item, count) in count_of_items {
        nodes.push(Node { data: item, count: count });
    }
    nodes
}

fn initialize_centroids<T: Data>(k: usize, nodes: &Vec<Node<T>>) -> Vec<Centroid<T>> {
    nodes.iter().take(k).map(|node| { node.to_centroid() }).collect()
}
