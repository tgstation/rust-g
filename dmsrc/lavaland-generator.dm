/**
 * Generates a procedural dungeon map using BSP tree partitioning, prefab placement,
 * MST corridor generation, and Cellular Automata smoothing.
 *
 * Returns a plain binary grid string (matching cellularnoise format):
 * A width*height string of '0' (wall) and '1' (floor) characters in row-major order.
 *
 * Arguments:
 * * width - Grid width (required)
 * * height - Grid height (required)
 * * prefabs_json - JSON array of prefab configs: [{"cx":55,"cy":65,"w":10,"h":10,"isEnclosed":true},...] (default: "[]")
 * * min_bsp_size - Minimum BSP leaf dimension (default: 20)
 * * max_ratio - Maximum aspect ratio for BSP splits (default: 2.5)
 * * padding - Room edge padding within BSP leaf (default: 2)
 * * room_fill_percent - How much of each BSP leaf a room fills, 1-100 (default: 50)
 * * corridor_width - Width of corridors between rooms (default: 1)
 * * loop_percent - Chance to add extra MST edges for loops (default: 15)
 * * noise_percent - Initial random floor density (default: 48)
 * * ca_steps - Cellular Automata smoothing iterations (default: 6)
 * * birth_limit - Neighbors to create floor (>=) (default: 5)
 * * survival_limit - Neighbors to survive as floor (>=) (default: 4)
 */
#define rustg_lavaland_generator_generate(width, height, prefabs_json, min_bsp_size, max_ratio, padding, room_fill_percent, corridor_width, loop_percent, noise_percent, ca_steps, birth_limit, survival_limit) \
	RUSTG_CALL(RUST_G, "lavaland_generator_generate")(width, height, prefabs_json, min_bsp_size, max_ratio, padding, room_fill_percent, corridor_width, loop_percent, noise_percent, ca_steps, birth_limit, survival_limit)
