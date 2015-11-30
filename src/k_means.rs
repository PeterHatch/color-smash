use std::collections::HashMap;
use std::collections::hash_state::DefaultState;
use std::fmt::Debug;
use std::hash::Hash;
use std::hash::SipHasher;

pub trait SimpleInput<O: Output> : Eq + Hash + Copy + Debug {
    fn distance_to(&self, other: &O) -> u64;
    fn as_output(&self) -> O;
    fn nearest(&self, centroids: &Vec<O>) -> u32 {
        centroids.iter().zip(0..).min_by(|&(centroid, _)| self.distance_to(centroid)).unwrap().1
    }
    fn count(&self) -> u32 {
        1
    }
}

pub trait Input<O: Output> : SimpleInput<O> {
    fn mean_of(data_and_counts: &Vec<&Self>) -> O;
}

pub trait Output : Eq + Hash + Copy + Debug {}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Grouped<T> {
    pub data: T,
    pub count: u32,
}

pub fn collect_groups<I, O>(items: I) -> Vec<Grouped<I::Item>>
    where I: Iterator,
          I::Item: SimpleInput<O>,
          O: Output {

    let mut count_of_items: HashMap<I::Item, u32, DefaultState<SipHasher>> = Default::default();
    for item in items {
        let counter = count_of_items.entry(item).or_insert(0);
        *counter += 1;
    }

    count_of_items.into_iter().map(|(item, count)| { Grouped { data: item, count: count } }).collect()
}

impl<T: SimpleInput<O>, O: Output> SimpleInput<O> for Grouped<T> {
    fn distance_to(&self, other: &O) -> u64 {
        self.data.distance_to(other)
    }
    fn as_output(&self) -> O {
        self.data.as_output()
    }
    fn count(&self) -> u32 {
        self.count
    }
}

pub fn quantize<I: Input<O>, O: Output>(nodes: &Vec<I>) -> (Vec<O>, Vec<Vec<&I>>) {
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

fn initialize_centroids<I: Input<O>, O: Output>(k: usize, nodes: &Vec<I>) -> Vec<O> {
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

fn find_nearest_centroids<'a, 'b, I: Input<O>, O: Output>(centroids: &'a Vec<O>, nodes: &'b Vec<I>) -> Vec<Vec<&'b I>> {
    let mut nodes_per_centroid = vec![Vec::new(); centroids.len()];

    for node in nodes {
        let centroid_index = node.nearest(centroids);
        nodes_per_centroid[centroid_index as usize].push(node);
    }

    nodes_per_centroid
}

fn reposition_centroids<I: Input<O>, O: Output>(centroids: &mut Vec<O>, nodes_per_centroid: &Vec<Vec<&I>>) {
    for (centroid, nodes) in centroids.iter_mut().zip(nodes_per_centroid.iter()) {
        *centroid = I::mean_of(nodes);
    }
}
