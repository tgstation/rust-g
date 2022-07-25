#define rustg_raw_read_toml_file(path) json_decode(call(RUST_G, "toml_file_to_json")(path) || "null")

/proc/rustg_read_toml_file(path)
	var/list/output = rustg_raw_read_toml_file(path)
	if (output["success"])
		return output["content"]
	else
		CRASH(output["content"])
