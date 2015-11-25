use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Data : Eq + Hash + Copy + Clone + Debug {
    type Output: Eq + Hash + Copy + Clone + Debug;
    fn distance_to(&self, other: &Self::Output) -> u64;
    fn mean_of(data_and_counts: &Vec<&Node<Self>>) -> Self::Output;
    fn as_output(&self) -> Self::Output;
}

#[derive(PartialEq, Eq, Debug)]
pub struct Node<T: Data> {
    pub data: T,
    pub count: u32,
}

impl<T: Data> Node<T> {
    fn to_centroid(&self) -> Centroid<T> {
        Centroid { data: self.data.as_output() }
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
    data: T::Output,
}

pub fn quantize<I>(items: I) -> HashMap<I::Item, <I::Item as Data>::Output>
    where I: Iterator,
          I::Item: Data {
    let k = 256;

    let nodes = create_nodes(items);
    println!("{:?} Nodes", nodes.len());
    let mut centroids = initialize_centroids(k, &nodes);

    let mut nodes_per_centroid = find_nearest_centroids(&centroids, &nodes);

    for iteration in 1.. {
        println!("Iteration {:?}", iteration);

        // Temp diagnostic message
        {
            let mut empty_centroids = 0;
            for node_list in &nodes_per_centroid {
                if node_list.is_empty() {
                    empty_centroids += 1;
                }
            }
            match empty_centroids {
                0 => (),
                1 => println!("1 centroid with no nodes found."),
                _ => println!("{} centroids with no nodes found.", empty_centroids),
            }
        }

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
    let mut centroids = Vec::with_capacity(k);
    let first_centroid = nodes.iter().max_by(|node| node.count).unwrap().to_centroid();
    centroids.push(first_centroid);

    let mut distance_per_node: Vec<_> = nodes.iter().map(|node| node.distance_to(&centroids[0])).collect();
    let mut centroid_per_node: Vec<_> = vec![0; nodes.len()];

    let mut distance_per_centroid: Vec<_> = Vec::with_capacity(k);
    let distance_to_first_centroid = distance_per_node.iter().sum();
    distance_per_centroid.push(distance_to_first_centroid);

    while centroids.len() < k {
        let centroid_to_split = distance_per_centroid.iter().zip(0..).max_by(|&(distance, _index)| distance).unwrap().1;
        let furthest_node_index = distance_per_node.iter().zip(centroid_per_node.iter()).zip(0..).filter(|&((_, centroid), _)| *centroid == centroid_to_split).max_by(|&((distance, _), _)| distance).unwrap().1;
        let new_centroid = nodes[furthest_node_index].to_centroid();

        if let Some(_) = centroids.iter().find(|&centroid| *centroid == new_centroid) {
            println!("Created duplicate centroid: {:?}", new_centroid);
        }

        let new_centroid_index = centroids.len();
        distance_per_centroid.push(0);

        for ((node, distance), centroid) in nodes.iter().zip(distance_per_node.iter_mut()).zip(centroid_per_node.iter_mut()) {
            let new_distance = node.distance_to(&new_centroid);
            if new_distance < *distance {
                distance_per_centroid[*centroid] -= *distance;
                *centroid = new_centroid_index;
                *distance = new_distance;
                distance_per_centroid[new_centroid_index] += new_distance;
            }
        }
        centroids.push(new_centroid);
    }

    centroids
}

fn find_nearest_centroids<'a, 'b, T: Data>(centroids: &'a Vec<Centroid<T>>, nodes: &'b Vec<Node<T>>) -> Vec<Vec<&'b Node<T>>> {
    let mut nodes_per_centroid = vec![Vec::new(); centroids.len()];

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

fn create_quantization_map<T: Data>(centroids: &Vec<Centroid<T>>, nodes_per_centroid: &Vec<Vec<&Node<T>>>) -> HashMap<T, T::Output> {
    let mut quantization_map = HashMap::new();

    for (centroid, nodes) in centroids.iter().zip(nodes_per_centroid.iter()) {
        for node in nodes {
            quantization_map.insert(node.data, centroid.data);
        }
    }

    quantization_map
}
