use crate::error::Result;
use rand::*;
use std::fmt::Write;

byond_fn! { cnoise_generate(percentage, smoothing_iterations, birth_limit, death_limit, width, height) {
    noise_gen(percentage, smoothing_iterations, birth_limit, death_limit, width, height).ok()
} }

fn noise_gen(
    percentage_as_str: &str,
    smoothing_level_as_str: &str,
    birth_limit_as_str: &str,
    death_limit_as_str: &str,
    width_as_str: &str,
    height_as_str: &str,
) -> Result<String> {
    let percentage = percentage_as_str.parse::<i32>()?;
    let smoothing_level = smoothing_level_as_str.parse::<i32>()?;
    let birth_limit = birth_limit_as_str.parse::<i32>()?;
    let death_limit = death_limit_as_str.parse::<i32>()?;
    let width = width_as_str.parse::<usize>()?;
    let height = height_as_str.parse::<usize>()?;

    // Noise generation
    let mut zplane = vec![vec![false; width]; height];
    for row in zplane.iter_mut() {
        for cell in row.iter_mut() {
            *cell = rand::thread_rng().gen_range(0..100) < percentage;
        }
    }

    // Smoothing part
    for _timer in 0..smoothing_level {
        let zplane_old = zplane.clone();
        for i in 0..height {
            for j in 0..width {
                let mut sum = 0;

                if i > 0 {
                    if j > 0 {
                        sum += if zplane_old[i - 1][j - 1] { 1 } else { 0 };
                    }

                    sum += if zplane_old[i - 1][j] { 1 } else { 0 };

                    if j + 1 < width {
                        sum += if zplane_old[i - 1][j + 1] { 1 } else { 0 };
                    }
                }

                if j > 0 {
                    sum += if zplane_old[i][j - 1] { 1 } else { 0 };
                }

                if j + 1 < width {
                    sum += if zplane_old[i][j + 1] { 1 } else { 0 };
                }

                if i + 1 < height {
                    if j > 0 {
                        sum += if zplane_old[i + 1][j - 1] { 1 } else { 0 };
                    }

                    sum += if zplane_old[i + 1][j] { 1 } else { 0 };

                    if j + 1 < width {
                        sum += if zplane_old[i + 1][j + 1] { 1 } else { 0 };
                    }
                }

                if zplane_old[i][j] {
                    if sum < death_limit {
                        zplane[i][j] = false;
                    } else {
                        zplane[i][j] = true;
                    }
                } else if sum > birth_limit {
                    zplane[i][j] = true;
                } else {
                    zplane[i][j] = false;
                }
            }
        }
    }

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
