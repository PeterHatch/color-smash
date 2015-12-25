use std::collections::HashMap;
use std::collections::hash_state::DefaultState;
use std::fmt::Debug;
use std::hash::Hash;
use std::hash::SipHasher;

use numeric_float::n64;

mod initializer;

pub trait SimpleInput : Eq + Hash + Clone + Debug {
    type Output: Output;

    fn distance_to(&self, other: &Self::Output) -> f64;
    fn normalized_distance(&self, other: &Self::Output) -> f64;
    fn as_output(&self) -> Self::Output;
    fn nearest(&self, centers: &Vec<Self::Output>) -> u32 {
        let centers_with_indexes = centers.iter().zip(0..);
        let (_center, cluster) = centers_with_indexes.min_by_key(|&(center, _cluster)| -> n64 {
                                                         self.distance_to(center).into()
                                                     })
                                                     .unwrap();
        cluster
    }
    fn count(&self) -> u32 {
        1
    }
}

pub trait Input : SimpleInput {
    fn mean_of(points: &Vec<&Self>) -> Self::Output;
}

pub trait Output : Eq + Hash + Clone + Debug {
    fn distance_to(&self, other: &Self) -> f64;
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Grouped<I: SimpleInput> {
    pub data: I,
    pub count: u32,
}

impl<I: SimpleInput> Grouped<I> {
    fn new(data: I, count: u32) -> Grouped<I> {
        Grouped {
            data: data,
            count: count,
        }
    }
}

pub fn collect_groups<I>(items: I) -> Vec<Grouped<I::Item>>
    where I: Iterator,
          I::Item: SimpleInput
{

    let mut count_of_items: HashMap<I::Item, u32, DefaultState<SipHasher>> = Default::default();
    for item in items {
        let counter = count_of_items.entry(item).or_insert(0);
        *counter += 1;
    }

    count_of_items.into_iter()
                  .map(|(item, count)| Grouped::new(item, count))
                  .collect()
}

impl<I: SimpleInput> SimpleInput for Grouped<I> {
    type Output = I::Output;

    fn distance_to(&self, other: &Self::Output) -> f64 {
        self.data.distance_to(other)
    }
    fn normalized_distance(&self, other: &Self::Output) -> f64 {
        self.data.normalized_distance(other)
    }
    fn as_output(&self) -> Self::Output {
        self.data.as_output()
    }
    fn count(&self) -> u32 {
        self.count
    }
}

pub fn run<I: Input>(data_points: &Vec<I>) -> (Vec<I::Output>, Vec<Vec<&I>>) {
    let k = 256;

    let (mut centers, mut points_per_cluster) = initializer::initialize_centers(k, &data_points);

    for iteration in 1.. {
        println!("Iteration {:?}", iteration);

        // Temp diagnostic message
        {
            let mut empty_clusters = 0;
            for points in &points_per_cluster {
                if points.is_empty() {
                    empty_clusters += 1;
                }
            }
            match empty_clusters {
                0 => (),
                1 => println!("1 empty cluster found."),
                _ => println!("{} empty clusters found.", empty_clusters),
            }
        }

        let prior_points_per_cluster = points_per_cluster.clone();
        reposition_centers(&mut centers, &points_per_cluster);
        points_per_cluster = assign_to_clusters(&centers, &points_per_cluster);

        if points_per_cluster == prior_points_per_cluster {
            break;
        }
    }

    (centers, points_per_cluster)
}

fn assign_to_clusters<'a, 'b, 'c, I>(centers: &'a Vec<I::Output>,
                                     prior_points_per_cluster: &'b Vec<Vec<&'c I>>)
                                     -> Vec<Vec<&'c I>>
    where I: Input
{
    let k = centers.len();
    let distances_between_centers = calculate_distances_between_centers(centers);
    let mut points_per_cluster = vec![Vec::new(); k];

    for i in 0..k {
        let points = &prior_points_per_cluster[i];
        let prior_center = &centers[i];
        let distances_to_other_centers = &distances_between_centers[i];

        for &point in points {
            let distance_to_prior_center = point.distance_to(&prior_center);
            let mut new_cluster = i as u32;
            let mut distance_to_new = distance_to_prior_center;

            for &(center_index, distance_between_centers) in distances_to_other_centers {
                if distance_to_prior_center * 4.0 <= distance_between_centers {
                    break;
                }
                let distance = point.distance_to(&centers[center_index as usize]);
                if distance < distance_to_new {
                    new_cluster = center_index;
                    distance_to_new = distance;
                    if distance_to_prior_center * 4.0 <= distance_between_centers {
                        println!("! distance to prior center: {}", distance_to_prior_center);
                        println!("  distance to new center: {}", distance);
                        println!("  distance between centers: {}", distance_between_centers);
                    }
                }
            }

            points_per_cluster[new_cluster as usize].push(point);
        }
    }

    points_per_cluster
}

fn calculate_distances_between_centers<O: Output>(centers: &Vec<O>) -> Vec<Vec<(u32, f64)>> {
    let k = centers.len();
    let mut distances_per_center = vec![Vec::with_capacity(k - 1); k];

    for i in 0..k {
        for j in 1..k {
            if i == j {
                continue;
            }

            let distance = centers[i].distance_to(&centers[j]);
            distances_per_center[i].push((j as u32, distance));
            distances_per_center[j].push((i as u32, distance));
        }
    }

    for distances in distances_per_center.iter_mut() {
        distances.sort_by_key(|&(_center_index, distance)| -> n64 { distance.into() });
    }

    distances_per_center
}

fn reposition_centers<I: Input>(centers: &mut Vec<I::Output>, points_per_cluster: &Vec<Vec<&I>>) {
    for (center, points) in centers.iter_mut().zip(points_per_cluster.iter()) {
        *center = I::mean_of(points);
    }
}
