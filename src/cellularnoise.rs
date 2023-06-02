use crate::error::Result;
use rand::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::fmt::Write;

byond_fn!(fn cnoise_generate(percentage, smoothing_iterations, birth_limit, death_limit, width, height) {
    noise_gen(percentage, smoothing_iterations, birth_limit, death_limit, width, height).ok()
});

fn noise_gen(
    percentage_as_str: &str,
    smoothing_level_as_str: &str,
    birth_limit_as_str: &str,
    death_limit_as_str: &str,
    width_as_str: &str,
    height_as_str: &str,
) -> Result<String> {
    let percentage = percentage_as_str.parse::<usize>()?;
    let smoothing_level = smoothing_level_as_str.parse::<usize>()?;
    let birth_limit = birth_limit_as_str.parse::<usize>()?;
    let death_limit = death_limit_as_str.parse::<usize>()?;
    let width = width_as_str.parse::<usize>()?;
    let height = height_as_str.parse::<usize>()?;
    //we populate it, from 0 to height+3, 0 to height+1 is exactly height long, but we also need border guards, so we add another +2, so it is 0..height+3
    let mut filled_vec = (0..width + 3)
        .into_par_iter()
        .map(|x| {
            let mut rng = rand::thread_rng();
            (0..height + 3)
                .map(|y| {
                    if x == 0 || y == 0 || x == width + 2 || y == height + 2 {
                        return false;
                    }
                    rng.gen_range(0..100) < percentage
                })
                .collect::<Vec<bool>>()
        })
        .collect::<Vec<Vec<bool>>>();

    //then we smoothe it
    (0..smoothing_level).for_each(|_| {
        let replace_vec = (0..width + 3)
            .into_par_iter()
            .map(|x| {
                (0..height + 3)
                    .map(|y| {
                        if x == 0 || y == 0 || x == width + 2 || y == height + 2 {
                            return false;
                        }
                        let sum: usize = filled_vec[x - 1][y - 1] as usize
                            + filled_vec[x - 1][y] as usize
                            + filled_vec[x - 1][y + 1] as usize
                            + filled_vec[x][y - 1] as usize
                            + filled_vec[x][y + 1] as usize
                            + filled_vec[x + 1][y - 1] as usize
                            + filled_vec[x + 1][y] as usize
                            + filled_vec[x + 1][y + 1] as usize;

                        if sum < death_limit && filled_vec[x][y] {
                            return false;
                        }
                        if sum > birth_limit && !filled_vec[x][y] {
                            return true;
                        }
                        filled_vec[x][y]
                    })
                    .collect::<Vec<bool>>()
            })
            .collect::<Vec<Vec<bool>>>();
        filled_vec = replace_vec;
    });

    //then we cut it
    let map = (1..=width)
        .into_par_iter()
        .map(|x| {
            (1..=height)
                .map(|y| filled_vec[x][y])
                .collect::<Vec<bool>>()
        })
        .collect::<Vec<Vec<bool>>>();

    let mut string = String::new();
    for row in map.iter() {
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
