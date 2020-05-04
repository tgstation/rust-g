use noise::{NoiseFn, Perlin, Seedable};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use crate::error::Result;

thread_local! {
    static GENERATORS: RefCell<HashMap<String,  Perlin>> = RefCell::new(HashMap::new());
}

byond_fn! {seed_noise_generator(generator_id, seed) {
    seed_generator(generator_id, seed).err()
} }

byond_fn! {get_noise_at_coordinates(generator_id, x, y){
    get_at_coordinates(generator_id, x, y).ok()
} }

fn seed_generator(generator_id: &str, seed_as_str: &str) -> Result<()> {
    let seed = seed_as_str.parse::<u32>()?;
    GENERATORS.with(|cell| {
        let mut generators = cell.borrow_mut();
        generators.insert(generator_id.to_string(), Perlin::new().set_seed(seed));
    });
    Ok(())
}

//note that this will be 0 at integer x & y, scaling is left up to the caller
fn get_at_coordinates(generator_id: &str, x_as_str: &str, y_as_str: &str) -> Result<String> {
    let x = x_as_str.parse::<f64>()?;
    let y = y_as_str.parse::<f64>()?;
    GENERATORS.with(|cell| {
        let generators = cell.borrow();
        let generator = generators.get(&generator_id.to_string());
        if let Some(generator) = generator {
            //perlin noise produces a result in [-sqrt(0.5), sqrt(0.5)] which we scale to [0, 1] for simplicity
            let unscaled = generator.get([x, y]);
            let scaled_from_0_to_1 = (unscaled * 2.0_f64.sqrt() + 1.0) / 2.0;
            Result::Ok(scaled_from_0_to_1.to_string())
        } else {
            Result::Err(crate::error::Error::Io(Error::new(
                ErrorKind::Other,
                "No such generator",
            )))
        }
    })
}
