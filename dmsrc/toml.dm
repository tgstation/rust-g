#define rustg_raw_read_toml_file(path) json_decode(call(RUST_G, "toml_file_to_json")(path) || "null")

/proc/rustg_read_toml_file(path)
	var/list/output = rustg_raw_read_toml_file(path)
	if (output["success"])
		return json_decode(output["content"])
	else
		CRASH(output["content"])

#define rustg_raw_toml_encode(json) json_decode(call(RUST_G, "toml_encode")(json))

/proc/rustg_toml_encode(value)
	var/list/output = rustg_raw_toml_encode(json_encode(value))
	if (output["success"])
		return output["content"]
	else
		CRASH(output["content"])
