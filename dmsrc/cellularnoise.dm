/**
 * This proc generates a cellular automata noise grid which can be used in procedural generation methods. Returns a single string that goes row by row, with values of 1 representing an alive cell, and a value of 0 representing a dead cell.
* * percentage (required) The chance of a turf starting closed
* * smoothing_iterations(required) The amount of iterations the cellular automata simulates before returning the results
* * birth_limit (required) If the number of neighboring cells is higher than this amount, a cell is born
* * death_limit (required) If the number of neighboring cells is lower than this amount, a cell dies
*/
#define rustg_cnoise_generate(percentage, smoothing_iterations, birth_limit, death_limit) call(RUST_G, "cnoise_generate")(percentage, smoothing_iterations, birth_limit, death_limit)
