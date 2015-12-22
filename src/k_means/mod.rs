use std::collections::HashMap;
use std::collections::hash_state::DefaultState;
use std::fmt::Debug;
use std::hash::Hash;
use std::hash::SipHasher;

use numeric_float::n64;

pub trait SimpleInput<O: Output> : Eq + Hash + Clone + Debug {
    fn distance_to(&self, other: &O) -> f64;
    fn normalized_distance(&self, other: &O) -> f64;
    fn as_output(&self) -> O;
    fn nearest(&self, centers: &Vec<O>) -> u32 {
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

pub trait Input<O: Output> : SimpleInput<O> {
    fn mean_of(points: &Vec<&Self>) -> O;
}

pub trait Output : Eq + Hash + Clone + Debug {
    fn distance_to(&self, other: &Self) -> f64;
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Grouped<T> {
    pub data: T,
    pub count: u32,
}

pub fn collect_groups<I, O>(items: I) -> Vec<Grouped<I::Item>>
    where I: Iterator,
          I::Item: SimpleInput<O>,
          O: Output
{

    let mut count_of_items: HashMap<I::Item, u32, DefaultState<SipHasher>> = Default::default();
    for item in items {
        let counter = count_of_items.entry(item).or_insert(0);
        *counter += 1;
    }

    count_of_items.into_iter()
                  .map(|(item, count)| {
                      Grouped {
                          data: item,
                          count: count,
                      }
                  })
                  .collect()
}

impl<T: SimpleInput<O>, O: Output> SimpleInput<O> for Grouped<T> {
    fn distance_to(&self, other: &O) -> f64 {
        self.data.distance_to(other)
    }
    fn normalized_distance(&self, other: &O) -> f64 {
        self.data.normalized_distance(other)
    }
    fn as_output(&self) -> O {
        self.data.as_output()
    }
    fn count(&self) -> u32 {
        self.count
    }
}

pub fn run<I: Input<O>, O: Output>(data_points: &Vec<I>) -> (Vec<O>, Vec<Vec<&I>>) {
    let k = 256;

    let (mut centers, mut points_per_cluster) = initialize_centers(k, &data_points);

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

fn initialize_centers<I: Input<O>, O: Output>(k: u32, points: &Vec<I>) -> (Vec<O>, Vec<Vec<&I>>) {
    let mut centers = Vec::with_capacity(k as usize);
    let first_center = points.iter().max_by_key(|point| point.count()).unwrap().as_output();
    centers.push(first_center);

    let mut distance_per_point: Vec<_> = points.iter()
                                               .map(|point| point.normalized_distance(&centers[0]))
                                               .collect();
    let mut cluster_per_point: Vec<_> = vec![0; points.len()];

    let mut distance_per_cluster: Vec<_> = Vec::with_capacity(k as usize);
    let distance_to_first_center = points.iter()
                                         .zip(distance_per_point.iter())
                                         .map(|(point, distance)| {
                                             distance * (point.count() as f64)
                                         })
                                         .sum();
    distance_per_cluster.push(distance_to_first_center);

    while centers.len() < (k as usize) {
        let cluster_to_split = worst_cluster(&distance_per_cluster);
        let farthest_point_index = farthest_point_of(cluster_to_split,
                                                     &cluster_per_point,
                                                     &distance_per_point);
        let new_center = points[farthest_point_index].as_output();

        if let Some(_) = centers.iter().find(|&center| *center == new_center) {
            println!("Created duplicate center: {:?}", new_center);
        }

        let new_cluster = centers.len();
        distance_per_cluster.push(0.0);

        for ((point, distance), cluster) in points.iter()
                                                  .zip(distance_per_point.iter_mut())
                                                  .zip(cluster_per_point.iter_mut()) {
            let new_distance = point.normalized_distance(&new_center);
            if new_distance < *distance {
                distance_per_cluster[*cluster] -= *distance * (point.count() as f64);
                *cluster = new_cluster;
                *distance = new_distance;
                distance_per_cluster[new_cluster] += new_distance * (point.count() as f64);
            }
        }
        centers.push(new_center);
    }

    let points_per_cluster = points_per_cluster(points, cluster_per_point, k);

    (centers, points_per_cluster)
}

fn points_per_cluster<I: Input<O>, O: Output>(points: &Vec<I>,
                                              cluster_per_point: Vec<usize>,
                                              k: u32)
                                              -> Vec<Vec<&I>> {
    let mut points_per_cluster = vec![Vec::new(); (k as usize)];

    for (point, cluster) in points.iter().zip(cluster_per_point.into_iter()) {
        points_per_cluster[cluster as usize].push(point);
    }

    points_per_cluster
}

fn worst_cluster(distance_per_cluster: &Vec<f64>) -> usize {
    let distances_with_indexes = distance_per_cluster.iter().zip(0..);
    let (_distance, cluster) =
        distances_with_indexes.max_by_key(|&(&distance, _cluster)| -> n64 { distance.into() })
                              .unwrap();
    cluster
}

fn farthest_point_of(target_cluster: usize,
                     cluster_per_point: &Vec<usize>,
                     distance_per_point: &Vec<f64>)
                     -> usize {
    let point_indexes = cluster_per_point.iter().zip(0..).filter_map(|(&cluster, point_index)| {
        if cluster == target_cluster {
            Some(point_index)
        } else {
            None
        }
    });
    let distances_and_indexes = point_indexes.map(|i| (distance_per_point[i], i));
    let (_distance, index) = distances_and_indexes.max_by_key(|&(distance, _index)| -> n64 {
                                                      distance.into()
                                                  })
                                                  .unwrap();
    index
}

fn assign_to_clusters<'a, 'b, 'c, I, O>(centers: &'b Vec<O>,
                                        prior_points_per_cluster: &'c Vec<Vec<&'a I>>)
                                        -> Vec<Vec<&'a I>>
    where I: Input<O>,
          O: Output
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

fn reposition_centers<I: Input<O>, O: Output>(centers: &mut Vec<O>,
                                              points_per_cluster: &Vec<Vec<&I>>) {
    for (center, points) in centers.iter_mut().zip(points_per_cluster.iter()) {
        *center = I::mean_of(points);
    }
}
