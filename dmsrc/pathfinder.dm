#define rustg_register_nodes(json) call(RUST_G, "register_nodes")(json)
/proc/rustg_nodes_len() return call(RUST_G, "nodes_len")()

