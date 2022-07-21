use crate::error::Result;
use core::panic;
use rand::prelude::*;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::fmt::Write;
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

byond_fn!(fn worley_generate(region_size, threshold, node_per_region_chance, size, node_min, node_max) {
    worley_noise(region_size, threshold, node_per_region_chance, size, node_min, node_max).ok()
});

const RANGE: usize = 4;

// This is a quite complex algorithm basically what it does is it creates 2 maps, one filled with cells and the other with 'regions' that map onto these cells.
// Each region can spawn 1 node, the cell then determines wether it is true or false depending on the distance from it to the nearest node in the region minus the second closest node.
// If this distance is greater than the threshold then the cell is true, otherwise it is false.
fn worley_noise(
    str_reg_size: &str,
    str_positive_threshold: &str,
    str_node_per_region_chance: &str,
    str_size: &str,
    str_node_min: &str,
    str_node_max: &str,
) -> Result<String> {
    let region_size = str_reg_size.parse::<i32>()?;
    let positive_threshold = str_positive_threshold.parse::<f32>()?;
    let size = str_size.parse::<usize>()?;
    let node_per_region_chance = str_node_per_region_chance.parse::<usize>()?;
    let node_min = str_node_min.parse::<u32>()?;
    let node_max = str_node_max.parse::<u32>()?;

    let world_size = (size as f32 / region_size as f32).ceil() as i32;

    let mut map = NoiseCellMap::new(region_size, world_size)
        .node_fill(node_min, node_max, node_per_region_chance)
        .worley_fill(positive_threshold)?;

    map.truncate(size);
    map.par_iter_mut().for_each(|row| {
        row.truncate(size);
    });

    let mut output = String::new();

    for row in map {
        for cell in row {
            if cell {
                let _ = write!(output, "1");
            } else {
                let _ = write!(output, "0");
            }
        }
    }
    Ok(output)
}
struct NoiseCellMap {
    reg_vec: Vec<Vec<NoiseCellRegion>>,
    reg_size: i32,
    reg_amt: i32,
}

impl NoiseCellMap {
    fn new(reg_size: i32, reg_amt: i32) -> Self {
        let mut noise_cell_map = NoiseCellMap {
            reg_vec: Vec::new(),
            reg_size,
            reg_amt,
        };
        for x in 0..reg_amt {
            noise_cell_map.reg_vec.push(Vec::new());
            for y in 0..reg_amt {
                noise_cell_map.reg_vec[x as usize].push(NoiseCellRegion::new((x, y), reg_size));
            }
        }
        noise_cell_map
    }

    fn node_fill(&mut self, mut node_min: u32, mut node_max: u32, node_chance: usize) -> &mut Self {
        node_min = node_min.max(1);
        node_max = node_min.max(node_max);
        let node_counter: Arc<RwLock<usize>> = Arc::new(RwLock::new(0));
        let reg_size = self.reg_size;
        self.reg_vec.par_iter_mut().flatten().for_each(|region| {
            let mut rng = ThreadRng::default();
            // basically, to make sure the algorithm works optimally or rather works at all without panicing we have to ensure we spawn at least some nodes
            // this amount of nodes scales inversely to range.
            {
                let mut write_guard = node_counter.write().unwrap();
                if (*write_guard < RANGE) && rng.gen_range(0..100) > node_chance {
                    *write_guard += 1;
                    return;
                }
                *write_guard = 0;
            }

            let amt = rng.gen_range(node_min..node_max);
            for _ in 0..amt {
                let coord = (rng.gen_range(0..reg_size), rng.gen_range(0..reg_size));
                region.insert_node(coord);
            }
        });
        self
    }

    fn get_nodes_in_range(&self, centre: (i32, i32), range: i32) -> HashSet<(i32, i32)> {
        let mut v = HashSet::new();
        for x in centre.0 - range..centre.0 + range {
            if x < 0 || x >= self.reg_amt {
                continue;
            }
            for y in centre.1 - range..centre.1 + range {
                if y < 0 || y >= self.reg_amt {
                    continue;
                }
                v.extend(
                    self.reg_vec[x as usize][y as usize]
                        .get_nodes()
                        .iter()
                        .cloned(),
                )
            }
        }
        v
    }

    fn worley_fill(&mut self, threshold: f32) -> Result<Vec<Vec<bool>>> {
        let new_data = self
            .reg_vec
            .par_iter()
            .flatten()
            .map(|region| {
                let mut edit_region = region.clone();
                let mut nodes_in_range =
                    self.get_nodes_in_range(region.reg_coordinates, RANGE as i32);
                {
                    let mut i = 1;
                    while nodes_in_range.len() < 2 {
                        i += 1;
                        nodes_in_range =
                            self.get_nodes_in_range(region.reg_coordinates, (i + RANGE) as i32);
                        if i > 32 {
                            panic!("Not enough nodes in range!");
                        }
                    }
                }
                for x in 0..region.reg_size {
                    for y in 0..region.reg_size {
                        edit_region.cell_vec[x as usize][y as usize] = (get_nth_smallest_dist(
                            region.to_global_coordinates((x, y)),
                            1,
                            &nodes_in_range,
                        ) - get_nth_smallest_dist(
                            region.to_global_coordinates((x, y)),
                            0,
                            &nodes_in_range,
                        )) > threshold;
                    }
                }
                edit_region
            })
            .collect::<Vec<NoiseCellRegion>>();
        let mut final_vec: Vec<Vec<bool>> = Vec::new();
        for x in 0..self.reg_amt * self.reg_size {
            final_vec.push(Vec::new());
            for _ in 0..self.reg_amt * self.reg_size {
                final_vec[x as usize].push(false);
            }
        }
        new_data.into_iter().for_each(|reg| {
            for x in 0..reg.reg_size {
                for y in 0..reg.reg_size {
                    let g_coords = reg.to_global_coordinates((x, y));
                    final_vec[g_coords.0 as usize][g_coords.1 as usize] =
                        reg.cell_vec[x as usize][y as usize];
                }
            }
        });
        Ok(final_vec)
    }
}
#[derive(Debug, Clone)]
struct NoiseCellRegion {
    cell_vec: Vec<Vec<bool>>,
    node_set: HashSet<(i32, i32)>,
    reg_coordinates: (i32, i32),
    reg_size: i32,
}

impl NoiseCellRegion {
    fn new(reg_coordinates: (i32, i32), reg_size: i32) -> Self {
        let mut noise_cell_region = NoiseCellRegion {
            cell_vec: Vec::new(),
            node_set: HashSet::new(),
            reg_coordinates,
            reg_size,
        };
        for x in 0..reg_size {
            noise_cell_region.cell_vec.push(Vec::new());
            for _ in 0..reg_size {
                noise_cell_region.cell_vec[x as usize].push(false);
            }
        }
        noise_cell_region
    }

    fn insert_node(&mut self, node: (i32, i32)) {
        self.node_set.insert(node);
    }

    fn to_global_coordinates(&self, coord: (i32, i32)) -> (i32, i32) {
        let mut c = (0, 0);
        c.0 = coord.0 + self.reg_coordinates.0 * self.reg_size;
        c.1 = coord.1 + self.reg_coordinates.1 * self.reg_size;
        c
    }

    fn get_nodes(&self) -> Vec<(i32, i32)> {
        self.node_set
            .clone()
            .into_iter()
            .map(|x| self.to_global_coordinates(x))
            .collect()
    }
}

fn sqr_distance(p1: (i32, i32), p2: (i32, i32)) -> f32 {
    (((p1.0 - p2.0).pow(2) + (p1.1 - p2.1).pow(2)) as f32).sqrt()
}

fn mht_distance(p1: (i32, i32), p2: (i32, i32)) -> f32 {
    ((p1.0 - p2.0).abs() + (p1.1 - p2.1).abs()) as f32
}

fn get_smallest_dist(centre: (i32, i32), set: &HashSet<(i32, i32)>) -> (i32, i32) {
    set.iter()
        .min_by(|a, b| {
            mht_distance(**a, centre)
                .partial_cmp(&mht_distance(**b, centre))
                .expect("Found NAN somehow")
        })
        .cloned()
        .expect("No minimum found")
}

fn get_nth_smallest_dist(centre: (i32, i32), mut nth: u32, set: &HashSet<(i32, i32)>) -> f32 {
    let mut our_set = set.clone();
    while nth > 0 && set.len() > 1 {
        our_set.remove(&get_smallest_dist(centre, &our_set));
        nth -= 1;
    }
    sqr_distance(centre, get_smallest_dist(centre, &our_set))
}
