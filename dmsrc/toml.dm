#define rustg_read_toml_file(path) json_decode(call(RUST_G, "toml_file_to_json")(path) || "null")
