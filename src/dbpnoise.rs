use crate::error::Result;
use dbpnoise::gen_noise;

byond_fn!(fn dbp_generate(seed, accuracy, stamp_size, world_size, lower_range, upper_range) {
    gen_dbp_noise(seed, accuracy, stamp_size, world_size, lower_range, upper_range).ok()
});

fn gen_dbp_noise(
    seed: &str,
    accuracy_as_str: &str,
    stamp_size_as_str: &str,
    world_size_as_str: &str,
    lower_range_as_str: &str,
    upper_range_as_str: &str,
) -> Result<String> {
    let accuracy = accuracy_as_str.parse::<usize>()?;
    let stamp_size = stamp_size_as_str.parse::<usize>()?;
    let world_size = world_size_as_str.parse::<usize>()?;
    let lower_range = lower_range_as_str.parse::<f32>()?;
    let upper_range = upper_range_as_str.parse::<f32>()?;
    let map: Vec<Vec<bool>> = gen_noise(
        seed,
        accuracy,
        stamp_size,
        world_size,
        lower_range,
        upper_range,
    );
    let mut result = String::with_capacity(world_size * world_size);
    for row in map {
        for cell in row {
            result.push(if cell { '1' } else { '0' });
        }
    }
    Ok(result)
}
