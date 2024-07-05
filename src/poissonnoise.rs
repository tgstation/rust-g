use fast_poisson::Poisson2D;
use std::fmt::Write;

use crate::error::Result;

byond_fn!(fn noise_poisson_map(seed, width, length, radius) {
    get_poisson_map(seed, width, length, radius).ok()
});

fn get_poisson_map(
    seed_as_str: &str,
    width_as_str: &str,
    length_as_str: &str,
    radius_as_str: &str,
) -> Result<String> {
    let width = width_as_str.parse::<f32>()?;
    let length = length_as_str.parse::<f32>()?;
    let radius = radius_as_str.parse::<f32>()?;
    let seed = seed_as_str.parse::<u64>()?;

    let points: Vec<[f32; 2]> = Poisson2D::new()
        .with_dimensions([width, length], radius)
        .with_seed(seed)
        .to_vec();

    let mut output = String::new();
    for y in 0..length as usize {
        for x in 0..width as usize {
            if points
                .iter()
                .any(|&point| point[0] as usize == x && point[1] as usize == y)
            {
                let _ = write!(output, "1");
            } else {
                let _ = write!(output, "0");
            }
        }
    }

    Ok(output)
}
