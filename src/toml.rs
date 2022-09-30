use crate::error::Result;

byond_fn!(fn toml_file_to_json(path) {
    serde_json::to_string(
        &match toml_file_to_json_impl(path) {
            Ok(value) => serde_json::json!({
                "success": true, "content": value
            }),
            Err(error) => serde_json::json!({
                "success": false, "content": error.to_string()
            }),
        }
    ).ok()
});

fn toml_file_to_json_impl(path: &str) -> Result<String> {
    Ok(serde_json::to_string(&toml_dep::from_str::<
        toml_dep::Value,
    >(&std::fs::read_to_string(
        path,
    )?)?)?)
}

byond_fn!(fn toml_file_from_json(value, path, prepend) {
    serde_json::to_string(
        &match toml_file_from_json_impl(value, path, prepend) {
            Ok(()) => "".to_owned(),
            Err(error) => error.to_string(),
        }
    ).ok()
});

fn toml_file_from_json_impl(value: &str, path: &str, prepend: &str) -> Result<()> {
    let toml_value: toml_dep::Value = serde_json::from_str(value)?;
    let mut toml_text = toml_dep::to_string(&toml_value)?;
    if !prepend.is_empty() {
        toml_text = format!("{prepend}\n{toml_text}");
    }

    std::fs::write(path, toml_text)?;

    Ok(())
}
