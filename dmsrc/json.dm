#define rustg_json_is_valid(text) (call(RUST_G, "json_is_valid")(text) == "true")
