use std::fs;

byond_fn! { toml_file2json(tomlfile) {
    let content = fs::read_to_string(tomlfile).unwrap_or("".to_string()); // If the file doesnt exist, just report back empty toml
    let val: toml::Value = toml::from_str(&content).unwrap_or(toml::from_str("").unwrap()); // If the toml is malformed, just report back empty json
    let json_data = serde_json::to_string(&val).unwrap();
    Some(json_data)
} }
