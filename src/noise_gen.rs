use noise::{NoiseFn, Perlin, Seedable};
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::error::Result;

thread_local! {
    static GENERATORS: RefCell<HashMap<String,  Perlin>> = RefCell::new(HashMap::new());
}

byond_fn! { noise_get_at_coordinates(seed, x, y) {
    get_at_coordinates(seed, x, y).ok()
} }

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
                let perlin = Perlin::new().set_seed(seed);
                vac.insert(perlin)
            }
        };
        //perlin noise produces a result in [-sqrt(0.5), sqrt(0.5)] which we scale to [0, 1] for simplicity
        let unscaled = generator.get([x, y]);
        let scaled_from_0_to_1 = (unscaled * 2.0_f64.sqrt() + 1.0) / 2.0;
        let clamped = if scaled_from_0_to_1 < 0.0 {
            0.0
        }
        else if scaled_from_0_to_1 > 1.0 {
            1.0
        }
        else {
            scaled_from_0_to_1
        };
        Ok(clamped.to_string())
    })
}
