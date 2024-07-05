#define rustg_noise_get_at_coordinates(seed, x, y) RUSTG_CALL(RUST_G, "noise_get_at_coordinates")(seed, x, y)

/**
 * params:
 * 	seed: str
 * 	x: int, width of the noisemap
 * 	y: int, height of the noisemap
 * 	radius: int, distance between points on the noisemap
 *
 * returns:
 * 	string: a X*Y length string of 1s and 0s representing a 2D poisson sample collapsed into a 1D string
 */
#define rustg_noise_poisson_sample(seed, x, y, r) RUSTG_CALL(RUST_G, "generate_poisson_sample")(seed, x, y, r)
