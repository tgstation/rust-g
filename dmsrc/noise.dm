#define rustg_noise_get_at_coordinates(seed, x, y) RUSTG_CALL(RUST_G, "noise_get_at_coordinates")(seed, x, y)
#define rustg_noise_poisson_sample(seed, x, y, r) RUSTG_CALL(RUST_G, "generate_poisson_sample")(seed, x, y, r)
