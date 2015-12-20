use std::collections::HashMap;
use std::collections::hash_state::DefaultState;
use std::fmt::Debug;
use std::hash::Hash;
use std::hash::SipHasher;

use numeric_float::n64;

pub trait SimpleInput<O: Output> : Eq + Hash + Clone + Debug {
    fn distance_to(&self, other: &O) -> f64;
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

pub trait Output : Eq + Hash + Clone + Debug {}

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
    fn as_output(&self) -> O {
        self.data.as_output()
    }
    fn count(&self) -> u32 {
        self.count
    }
}

pub fn run<I: Input<O>, O: Output>(data_points: &Vec<I>) -> (Vec<O>, Vec<Vec<&I>>) {
    let k = 256;

    let mut centers = initialize_centers(k, &data_points);
    let mut points_per_cluster = assign_to_clusters(&data_points, &centers);

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
        points_per_cluster = assign_to_clusters(&data_points, &centers);

        if points_per_cluster == prior_points_per_cluster {
            break;
        }
    }

    (centers, points_per_cluster)
}

fn initialize_centers<I: Input<O>, O: Output>(k: usize, points: &Vec<I>) -> Vec<O> {
    let mut centers = Vec::with_capacity(k);
    let first_center = points.iter().max_by_key(|point| point.count()).unwrap().as_output();
    centers.push(first_center);

    let mut distance_per_point: Vec<_> = points.iter()
                                               .map(|point| point.distance_to(&centers[0]))
                                               .collect();
    let mut cluster_per_point: Vec<_> = vec![0; points.len()];

    let mut distance_per_cluster: Vec<_> = Vec::with_capacity(k);
    let distance_to_first_center = points.iter()
                                         .zip(distance_per_point.iter())
                                         .map(|(point, distance)| {
                                             distance * (point.count() as f64)
                                         })
                                         .sum();
    distance_per_cluster.push(distance_to_first_center);

    while centers.len() < k {
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
            let new_distance = point.distance_to(&new_center);
            if new_distance < *distance {
                distance_per_cluster[*cluster] -= *distance * (point.count() as f64);
                *cluster = new_cluster;
                *distance = new_distance;
                distance_per_cluster[new_cluster] += new_distance * (point.count() as f64);
            }
        }
        centers.push(new_center);
    }

    centers
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

fn assign_to_clusters<'a, 'b, I: Input<O>, O: Output>(points: &'a Vec<I>,
                                                      centers: &'b Vec<O>)
                                                      -> Vec<Vec<&'a I>> {
    let mut points_per_cluster = vec![Vec::new(); centers.len()];

    for point in points {
        let cluster = point.nearest(centers);
        points_per_cluster[cluster as usize].push(point);
    }

    points_per_cluster
}

fn reposition_centers<I: Input<O>, O: Output>(centers: &mut Vec<O>,
                                              points_per_cluster: &Vec<Vec<&I>>) {
    for (center, points) in centers.iter_mut().zip(points_per_cluster.iter()) {
        *center = I::mean_of(points);
    }
}
