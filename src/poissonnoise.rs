use crate::error::Result;
use fast_poisson::Poisson2D;
use std::collections::HashSet;

byond_fn!(fn noise_poisson_map(seed, width, length, radius) {
    get_poisson_map(seed, width, length, radius).ok()
});

fn get_poisson_map(
    seed_as_str: &str,
    width_as_str: &str,
    length_as_str: &str,
    radius_as_str: &str,
) -> Result<String> {
    let width = width_as_str.parse::<usize>()?;
    let length = length_as_str.parse::<usize>()?;
    let radius = radius_as_str.parse::<f32>()?;
    let seed = seed_as_str.parse::<u64>()?;

    let points: HashSet<(usize, usize)> = Poisson2D::new()
        .with_dimensions([width as f32, length as f32], radius)
        .with_seed(seed)
        .iter()
        .map(|[x, y]| (x as usize, y as usize))
        .collect();

    let mut output = String::with_capacity(width * length);
    for y in 0..length {
        for x in 0..width {
            if points.contains(&(x, y)) {
                output.push('1');
            } else {
                output.push('0');
            }
        }
    }

    Ok(output)
}
