use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Data : Eq + Hash + Copy + Clone + Debug {
    fn distance_to(&self, other: &Self) -> u64;
    fn mean_of(data_and_counts: &Vec<Node<Self>>) -> Self;
}

#[derive(Debug)]
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

    fn nearest<'a, 'b>(&'a self, centroids: &'b Vec<Centroid<T>>) -> &'b Centroid<T> {
        centroids.iter().min_by(|centroid| self.distance_to(centroid)).unwrap()
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

    let nearest_centroid_map = find_nearest_centroids(&centroids, &nodes);

    println!("{:?}", nodes.len());
    println!("{:?}", centroids.len());

    create_quantization_map(&nearest_centroid_map)
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

fn find_nearest_centroids<'a, 'b, T: Data>(centroids: &'a Vec<Centroid<T>>, nodes: &'b Vec<Node<T>>) -> HashMap<&'a Centroid<T>, Vec<&'b Node<T>>> {
    let mut nearest_centroid_map = HashMap::with_capacity(centroids.len());

    for centroid in centroids {
        nearest_centroid_map.insert(centroid, Vec::new());
    }

    for node in nodes {
        let centroid = node.nearest(centroids);
        nearest_centroid_map.get_mut(centroid).unwrap().push(node);
    }

    nearest_centroid_map
}

fn create_quantization_map<T: Data>(nearest_centroid_map: &HashMap<&Centroid<T>, Vec<&Node<T>>>) -> HashMap<T, T> {
    let mut quantization_map = HashMap::new();

    for (centroid, nodes) in nearest_centroid_map {
        for node in nodes {
            quantization_map.insert(node.data, centroid.data);
        }
    }

    quantization_map
}
