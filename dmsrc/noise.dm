#define rustg_noise_get_at_coordinates(seed, x, y) RUSTG_CALL(RUST_G, "noise_get_at_coordinates")(seed, x, y)

/**
 * Generates a 2D poisson disk distribution ('blue noise'), which is relatively uniform.
 *
 * params:
 * 	`seed`: str
 * 	`width`: int, width of the noisemap (see world.maxx)
 * 	`length`: int, height of the noisemap (see world.maxy)
 * 	`radius`: int, distance between points on the noisemap
 *
 * returns:
 * 	a width*length length string of 1s and 0s representing a 2D poisson sample collapsed into a 1D string
 */
#define rustg_noise_poisson_map(seed, width, length, radius) RUSTG_CALL(RUST_G, "noise_poisson_map")(seed, width, length, radius)
