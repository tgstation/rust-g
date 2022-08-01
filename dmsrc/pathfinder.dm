#define rustg_register_nodes(json) call(RUST_G, "register_nodes")(json)
#define rustg_astar_generate_path(start_node_id, goal_node_id) call(RUST_G, "astar_generate_path")(start_node_id, goal_node_id)
