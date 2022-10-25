#define rustg_json_is_valid(text) (RGCALL(RUST_G, "json_is_valid")(text) == "true")
