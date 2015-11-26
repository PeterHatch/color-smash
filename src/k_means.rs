use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub trait SimpleInput : Eq + Hash + Copy + Debug {
    type Output: Output;
    fn distance_to(&self, other: &Self::Output) -> u64;
    fn as_output(&self) -> Self::Output;
    fn nearest(&self, centroids: &Vec<Self::Output>) -> u32 {
        centroids.iter().zip(0..).min_by(|&(centroid, _)| self.distance_to(centroid)).unwrap().1
    }
    fn count(&self) -> u32 {
        1
    }
}

pub trait Input : SimpleInput {
    fn mean_of(data_and_counts: &Vec<&Self>) -> Self::Output;
}

pub trait Output : Eq + Hash + Copy + Debug {}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Grouped<T: SimpleInput> {
    pub data: T,
    pub count: u32,
}

pub fn collect_groups<I>(items: I) -> Vec<Grouped<I::Item>>
    where I: Iterator,
          I::Item: SimpleInput {

    let mut count_of_items: HashMap<I::Item, u32> = HashMap::new();
    for item in items {
        let counter = count_of_items.entry(item).or_insert(0);
        *counter += 1;
    }

    count_of_items.into_iter().map(|(item, count)| { Grouped { data: item, count: count } }).collect()
}

impl<T: SimpleInput> SimpleInput for Grouped<T> {
    type Output = T::Output;
    fn distance_to(&self, other: &Self::Output) -> u64 {
        self.data.distance_to(other)
    }
    fn as_output(&self) -> Self::Output {
        self.data.as_output()
    }
    fn count(&self) -> u32 {
        self.count
    }
}

pub fn quantize<T: Input>(nodes: &Vec<T>) -> (Vec<T::Output>, Vec<Vec<&T>>) {
    let k = 256;

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

    (centroids, nodes_per_centroid)
}

fn initialize_centroids<T: Input>(k: usize, nodes: &Vec<T>) -> Vec<T::Output> {
    let mut centroids = Vec::with_capacity(k);
    let first_centroid = nodes.iter().max_by(|node| node.count()).unwrap().as_output();
    centroids.push(first_centroid);

    let mut distance_per_node: Vec<_> = nodes.iter().map(|node| node.distance_to(&centroids[0])).collect();
    let mut centroid_per_node: Vec<_> = vec![0; nodes.len()];

    let mut distance_per_centroid: Vec<_> = Vec::with_capacity(k);
    let distance_to_first_centroid = nodes.iter().zip(distance_per_node.iter()).map(|(node, distance)| distance * (node.count() as u64)).sum();
    distance_per_centroid.push(distance_to_first_centroid);

    while centroids.len() < k {
        let centroid_to_split = distance_per_centroid.iter().zip(0..).max_by(|&(distance, _index)| distance).unwrap().1;
        let furthest_node_index = distance_per_node.iter().zip(centroid_per_node.iter()).zip(0..).filter(|&((_, centroid), _)| *centroid == centroid_to_split).max_by(|&((distance, _), _)| distance).unwrap().1;
        let new_centroid = nodes[furthest_node_index].as_output();

        if let Some(_) = centroids.iter().find(|&centroid| *centroid == new_centroid) {
            println!("Created duplicate centroid: {:?}", new_centroid);
        }

        let new_centroid_index = centroids.len();
        distance_per_centroid.push(0);

        for ((node, distance), centroid) in nodes.iter().zip(distance_per_node.iter_mut()).zip(centroid_per_node.iter_mut()) {
            let new_distance = node.distance_to(&new_centroid);
            if new_distance < *distance {
                distance_per_centroid[*centroid] -= *distance * (node.count() as u64);
                *centroid = new_centroid_index;
                *distance = new_distance;
                distance_per_centroid[new_centroid_index] += new_distance * (node.count() as u64);
            }
        }
        centroids.push(new_centroid);
    }

    centroids
}

fn find_nearest_centroids<'a, 'b, T: Input>(centroids: &'a Vec<T::Output>, nodes: &'b Vec<T>) -> Vec<Vec<&'b T>> {
    let mut nodes_per_centroid = vec![Vec::new(); centroids.len()];

    for node in nodes {
        let centroid_index = node.nearest(centroids);
        nodes_per_centroid[centroid_index as usize].push(node);
    }

    nodes_per_centroid
}

fn reposition_centroids<T: Input>(centroids: &mut Vec<T::Output>, nodes_per_centroid: &Vec<Vec<&T>>) {
    for (centroid, nodes) in centroids.iter_mut().zip(nodes_per_centroid.iter()) {
        *centroid = T::mean_of(nodes);
    }
}
