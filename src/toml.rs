use crate::error::Result;

byond_fn!(fn toml_file_to_json(path) {
    toml_file_to_json_impl(path).ok()
});

fn toml_file_to_json_impl(path: &str) -> Result<String> {
    Ok(serde_json::to_string(&toml_dep::from_str::<
        toml_dep::Value,
    >(&std::fs::read_to_string(
        path,
    )?)?)?)
}
