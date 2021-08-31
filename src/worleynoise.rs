use crate::error::Result;
use rand::*;
use std::fmt::Write;

byond_fn! { worley_generate(density, threshold, width, height) {
    worley_noise(density, threshold, width, height).ok()
} }

//In the worst possible situation this is O(n^3) in the best possible O(n^2.5), so just keep it in mind.
fn worley_noise(    density_as_str: &str,
    positive_threshold_as_str: &str,
    width_as_str: &str,
    height_as_str: &str,
) -> Result<String> {
    let density = density_as_str.parse::<f64>()?; // density of noise, 0 means no nodes and 100 means that every tile has a node.
    let positive_threshold = positive_threshold_as_str.parse::<f64>()?; // threshold, if value in cell is above this it gets set to true, otherwise false.
    let width = width_as_str.parse::<usize>()?;
    let height = height_as_str.parse::<usize>()?;

    let mut rng = rand::thread_rng();

    let mut zplane = vec![vec![false; width]; height];
    let mut node_vec = Vec::new();

    //we generate a node density map
    while node_vec.len() < 2 {
        for row in 0..width as i32 {
            for cell in 0..height as i32 {
                if rng.gen_range(0..100) as f64 <= density {
                    let node = WorleyNode::new(row, cell);
                    node_vec.push(node);

                }
            }
        }
    }

    //we generate the actual noise by comparing the distance to the nearest node to the distance of the second nearest node and checking if it passes the threshold
    for row in 0..width as i32{
        for cell in 0..height as i32 {
            dmsort::sort_by(&mut node_vec, |a,b| a.distance_to_sqrt(&cell,&row).partial_cmp( &b.distance_to_sqrt(&cell,&row)).unwrap());
            let comparative_distance = (node_vec[0].distance_to_sqrt(&cell,&row) - node_vec[1].distance_to_sqrt(&cell,&row)).abs();
            if comparative_distance > positive_threshold {
                zplane[cell as usize][row as usize] = true;
            }
        }
    }

    //we write it to a string and spit it back out
    let mut string = String::new();
    for row in zplane.iter() {
        for cell in row.iter() {
            if *cell {
                let _ = write!(string, "1");
            } else {
                let _ = write!(string, "0");
            }

        }
    }

   Ok(string)
}


struct WorleyNode{
    x: i32,
    y: i32,
}

impl WorleyNode {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y}
    }

    fn distance_to_sqrt(&self, other_x: &i32, other_y: &i32) -> f64 {
        let x_diff = self.x - other_x;
        let y_diff = self.y - other_y;
        f64::from(x_diff * x_diff + y_diff * y_diff).sqrt()
    }
}
