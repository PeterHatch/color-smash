use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Data : Eq + Hash + Copy + Clone + Debug {
    fn distance_to(&self, other: &Self) -> u64;
    fn mean_of(data_and_counts: &Vec<&Node<Self>>) -> Self;
}

#[derive(PartialEq, Eq, Debug)]
pub struct Node<T: Data> {
    pub data: T,
    pub count: u32,
}

impl<T: Data> Node<T> {
    fn to_centroid(&self) -> Centroid<T> {
        Centroid { data: self.data }
    }

    fn distance_to(&self, centroid: &Centroid<T>) -> u64 {
        self.data.distance_to(&centroid.data)
    }

    fn nearest(&self, centroids: &Vec<Centroid<T>>) -> u32 {
        centroids.iter().zip(0..).min_by(|&(centroid, _)| self.distance_to(centroid)).unwrap().1
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct Centroid<T: Data> {
    data: T,
}

pub fn quantize<I>(items: I) -> HashMap<I::Item, I::Item>
    where I: Iterator,
          I::Item: Data {
    let k = 256;

    let nodes = create_nodes(items);
    let mut centroids = initialize_centroids(k, &nodes);

    let mut nodes_per_centroid = find_nearest_centroids(&centroids, &nodes);

    loop {
        let last_nodes_per_centroid = nodes_per_centroid.clone();
        reposition_centroids(&mut centroids, &nodes_per_centroid);
        nodes_per_centroid = find_nearest_centroids(&centroids, &nodes);

        if nodes_per_centroid == last_nodes_per_centroid {
            break;
        }
    }

    create_quantization_map(&centroids, &nodes_per_centroid)
}

fn create_nodes<I>(items: I) -> Vec<Node<I::Item>>
    where I: Iterator,
          I::Item: Data {

    let mut count_of_items: HashMap<I::Item, u32> = HashMap::new();
    for item in items {
        let counter = count_of_items.entry(item).or_insert(0);
        *counter += 1;
    }

    count_of_items.into_iter().map(|(item, count)| { Node { data: item, count: count } }).collect()
}

fn initialize_centroids<T: Data>(k: usize, nodes: &Vec<Node<T>>) -> Vec<Centroid<T>> {
    nodes.iter().take(k).map(|node| { node.to_centroid() }).collect()
}

fn find_nearest_centroids<'a, 'b, T: Data>(centroids: &'a Vec<Centroid<T>>, nodes: &'b Vec<Node<T>>) -> Vec<Vec<&'b Node<T>>> {
    let mut nodes_per_centroid = Vec::with_capacity(centroids.len());

    for _ in 0..centroids.len() {
        nodes_per_centroid.push(Vec::new());
    }

    for node in nodes {
        let centroid_index = node.nearest(centroids);
        nodes_per_centroid[centroid_index as usize].push(node);
    }

    nodes_per_centroid
}

fn reposition_centroids<T: Data>(centroids: &mut Vec<Centroid<T>>, nodes_per_centroid: &Vec<Vec<&Node<T>>>) {
    for (centroid, nodes) in centroids.iter_mut().zip(nodes_per_centroid.iter()) {
        centroid.data = T::mean_of(nodes);
    }
}

fn create_quantization_map<T: Data>(centroids: &Vec<Centroid<T>>, nodes_per_centroid: &Vec<Vec<&Node<T>>>) -> HashMap<T, T> {
    let mut quantization_map = HashMap::new();

    for (centroid, nodes) in centroids.iter().zip(nodes_per_centroid.iter()) {
        for node in nodes {
            quantization_map.insert(node.data, centroid.data);
        }
    }

    quantization_map
}
