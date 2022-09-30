#define rustg_raw_read_toml_file(path) json_decode(call(RUST_G, "toml_file_to_json")(path) || "null")

/proc/rustg_read_toml_file(path)
	var/list/output = rustg_raw_read_toml_file(path)
	if (output["success"])
		return json_decode(output["content"])
	else
		CRASH(output["content"])

#define rustg_raw_write_toml_file(value, path, prepend) json_decode(call(RUST_G, "toml_file_from_json")(json_encode(value), path, prepend) || "null")

/proc/rustg_write_toml_file(value, path, prepend = "")
	var/error = rustg_raw_write_toml_file(json_encode(value), path, prepend)
	if (error)
		CRASH(error)
