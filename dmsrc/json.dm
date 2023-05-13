#define rustg_json_is_valid(text) (RUSTG_CALL(RUST_G, "json_is_valid")(text) == "true")
