use super::{Input, Output};
use numeric_float::n64;

pub fn initialize_centers<I: Input<O>, O: Output>(k: u32,
                                                  points: &Vec<I>)
                                                  -> (Vec<O>, Vec<Vec<&I>>) {
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
