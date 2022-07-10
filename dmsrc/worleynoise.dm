/**
 * This proc generates a noise grid using worley noise algorithm
 *
 * Returns a single string that goes row by row, with values of 1 representing an alive cell, and a value of 0 representing a dead cell.
 *
 * Arguments:
 * * region_size: The size of regions
 * * threshold: the value that determines wether a cell is dead or alive
 * * node_per_region_chance: chance of a node existiing in a region
 * * size: size of the returned grid
 * * node_min: minimum amount of nodes in a region (after the node_per_region_chance is applied)
 * * node_max: maximum amount of nodes in a region
 */
#define rustg_worley_generate(region_size, threshold, node_per_region_chance, size, node_min, node_max) \
	call(RUST_G, "worley_generate")(region_size, threshold, node_per_region_chance, size, node_min, node_max)

