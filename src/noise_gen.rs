use std::collections::HashMap;
use std::cell::RefCell;
use noise::{Perlin, Seedable, NoiseFn};
use std::io::{BufWriter, Write, Error, ErrorKind};
use std::fs::File;

use crate::error::Result;


thread_local! {
    static GENERATORS: RefCell<HashMap<String,  Perlin>> = RefCell::new(HashMap::new());
}

byond_fn! {perlin_noise_2d_file(filename, seed, scaling) {
    make_noise_file(filename, seed, scaling).err()
} }

byond_fn! {seed_noise_generator(generator_id, seed) {
    seed_generator(generator_id, seed).err()
} }

byond_fn! {get_noise_at_coordinates(generator_id, x, y){
    get_at_coordinates(generator_id, x, y).ok()  
} }

fn seed_generator(generator_id: &str, seed_as_str: &str) -> Result<()>{
    let seed = seed_as_str.parse::<u32>()?;
    GENERATORS.with(|cell|{
        let mut generators = cell.borrow_mut();
        generators.insert(generator_id.to_string(), Perlin::new().set_seed(seed));
    });
    Ok(())
}

//note that this will be 0 at integer x & y, scaling is left up to the caller
fn get_at_coordinates(generator_id: &str, x_as_str: &str, y_as_str: &str) -> Result<String>{
    let x = x_as_str.parse::<f64>()?;
    let y = y_as_str.parse::<f64>()?;
    GENERATORS.with(|cell|{
        let generators = cell.borrow();
        let generator = generators.get(&generator_id.to_string());
        if let Some(generator) = generator{
            //perlin noise produces a result in [-sqrt(0.5), sqrt(0.5)] which we scale to [0, 1] for simplicity
            let unscaled = generator.get([x,y]);
            let scaled_from_0_to_1 = (unscaled * 2.0.sqrt() + 1)/2.0;
            Result::Ok(scaled_from_0_to_1.to_string()))
        }
        else{
            Result::Err(crate::error::Error::Io(Error::new(ErrorKind::Other, "No such generator")))
        }
    })
}

//outputs a 255*255 noise file, with rows seperated by newlines and columns separated by commas
fn make_noise_file(filename: &str, seed_as_str: &str, scaling_as_str: &str) -> Result<()> {
    let seed = seed_as_str.parse::<u32>()?;
    let scaling = scaling_as_str.parse::<f64>()?;
    let mut file = BufWriter::new(File::create(filename)?);
    let noise = Perlin::new().set_seed(seed);
    for y in 0..255{
        let row_string = (0u32..255u32).map(|x|noise.get([x as f64*scaling,y as f64*scaling]))
              .map(|noise|noise.to_string())
              .collect::<Vec<String>>()
              .join(",");

        write!(&mut file, "{}", row_string)?;
        file.write(b"\n")?;
    }
    file.flush()?;
    Ok(())
}
