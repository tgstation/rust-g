/**
 * Generates a procedural dungeon map using BSP tree partitioning, prefab placement,
 * MST corridor generation, and Cellular Automata smoothing.
 *
 * Returns a plain binary grid string (matching cellularnoise format):
 * A width*height string of '0' (wall) and '1' (floor) characters in row-major order.
 *
 * Arguments:
 * * width - Grid width
 * * height - Grid height
 * * prefabs_json - JSON array of prefab configs: [{"x":55,"y":65,"w":10,"h":10,"isEnclosed":true},...] (if none use "[]") x = bottom-left turf x, y = bottom-left turf y, w = prefab width, h = prefab height, isEnclosed = whether prefab should be treated like its wall or floor by the generation
 * * min_bsp_size - Minimum BSP leaf dimension
 * * max_ratio - Maximum aspect ratio for BSP splits
 * * padding - Room edge padding within BSP leaf
 * * room_fill_percent - How much of each BSP leaf a room fills, 1-100
 * * corridor_width - Width of corridors between rooms
 * * loop_percent - Chance to add extra MST edges for loops
 * * noise_percent - Initial random floor density
 * * ca_steps - Cellular Automata smoothing iterations
 * * birth_limit - Neighbors to create floor (>=)
 * * survival_limit - Neighbors to survive as floor (>=)
 * * edge_is_alive - Whether out-of-bounds cells count as ALIVE (floor) for CA neighbor counts
 */
#define rustg_lavaland_generator_generate(width, height, prefabs_json, min_bsp_size, max_ratio, padding, room_fill_percent, corridor_width, loop_percent, noise_percent, ca_steps, birth_limit, survival_limit, edge_is_alive) \
	RUSTG_CALL(RUST_G, "lavaland_generator_generate")(width, height, prefabs_json, min_bsp_size, max_ratio, padding, room_fill_percent, corridor_width, loop_percent, noise_percent, ca_steps, birth_limit, survival_limit, edge_is_alive)
