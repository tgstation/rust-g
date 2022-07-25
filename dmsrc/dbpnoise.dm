
/**
 * This proc generates a grid of perlin-like noise
 *
 * Returns a single string that goes row by row, with values of 1 representing an turned on cell, and a value of 0 representing a turned off cell.
 *
 * Arguments:
 * * seed: seed for the function
 * * accuracy: how close this is to the original perlin noise, as accuracy approaches infinity, the noise becomes more and more perlin-like
 * * stamp_size: Size of a singular stamp used by the algorithm, think of this as the same stuff as frequency in perlin noise
 * * world_size: size of the returned grid.
 * * lower_range: lower bound of values selected for. (inclusive)
 * * upper_range: upper bound of values selected for. (exclusive)
 */
#define rustg_dbp_generate(seed, accuracy, stamp_size, world_size, lower_range, upper_range) \
	call(RUST_G, "dbp_generate")(seed, accuracy, stamp_size, world_size, lower_range, upper_range)

