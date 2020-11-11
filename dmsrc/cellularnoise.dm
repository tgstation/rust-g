#define rustg_cnoise_generate(precentage, smoothing_iterations, birth_limit, death_limit) call(RUST_G, "cnoise_generate")(precentage, smoothing_iterations, birth_limit, death_limit)
#define rustg_cnoise_get_at_coordinates(grid,xcord,ycord) call(RUST_G, "cnoise_get_at_coordinates")(grid,xcord,ycord)
