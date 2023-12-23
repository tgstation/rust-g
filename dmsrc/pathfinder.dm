/**
 * Register a list of nodes into a rust library. This list of nodes must have been serialized in a json.
 * Node {// Index of this node in the list of nodes
 *  	  unique_id: usize,
 *  	  // Position of the node in byond
 *  	  x: usize,
 *  	  y: usize,
 *  	  z: usize,
 *  	  // Indexes of nodes connected to this one
 *  	  connected_nodes_id: Vec<usize>}
 * It is important that the node with the unique_id 0 is the first in the json, unique_id 1 right after that, etc.
 * It is also important that all unique ids follow. {0, 1, 2, 4} is not a correct list and the registering will fail
 * Nodes should not link across z levels.
 * A node cannot link twice to the same node and shouldn't link itself either
 */
#define rustg_register_nodes_astar(json) RUSTG_CALL(RUST_G, "register_nodes_astar")(json)

/**
 * Add a new node to the static list of nodes. Same rule as registering_nodes applies.
 * This node unique_id must be equal to the current length of the static list of nodes
 */
#define rustg_add_node_astar(json) RUSTG_CALL(RUST_G, "add_node_astar")(json)

/**
 * Remove every link to the node with unique_id. Replace that node by null
 */
#define rustg_remove_node_astar(unique_id) RUSTG_CALL(RUST_G, "remove_node_astar")(unique_id)

/**
 * Compute the shortest path between start_node and goal_node using A*. Heuristic used is simple geometric distance
 */
#define rustg_generate_path_astar(start_node_id, goal_node_id) RUSTG_CALL(RUST_G, "generate_path_astar")(start_node_id, goal_node_id)
