use fast_poisson::Poisson2D;
use noise::{NoiseFn, Perlin};
use std::{
    cell::RefCell,
    collections::hash_map::{Entry, HashMap},
    fmt::Write,
};

use crate::error::Result;

thread_local! {
    static GENERATORS: RefCell<HashMap<String,  Perlin>> = RefCell::new(HashMap::new());
}

byond_fn!(fn noise_get_at_coordinates(seed, x, y) {
    get_at_coordinates(seed, x, y).ok()
});

//note that this will be 0 at integer x & y, scaling is left up to the caller
fn get_at_coordinates(seed_as_str: &str, x_as_str: &str, y_as_str: &str) -> Result<String> {
    let x = x_as_str.parse::<f64>()?;
    let y = y_as_str.parse::<f64>()?;
    GENERATORS.with(|cell| {
        let mut generators = cell.borrow_mut();
        let mut entry = generators.entry(seed_as_str.to_string());
        let generator = match entry {
            Entry::Occupied(ref mut occ) => occ.get_mut(),
            Entry::Vacant(vac) => {
                let seed = seed_as_str.parse::<u32>()?;
                let perlin = Perlin::new(seed);
                vac.insert(perlin)
            }
        };
        //perlin noise produces a result in [-sqrt(0.5), sqrt(0.5)] which we scale to [0, 1] for simplicity
        let unscaled = generator.get([x, y]);
        let scaled = (unscaled * 2.0_f64.sqrt() + 1.0) / 2.0;
        let clamped = scaled.clamp(0.0, 1.0);
        Ok(clamped.to_string())
    })
}

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
