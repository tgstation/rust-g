byond_fn! { json_is_valid(text) {
	Some(
        serde_json::from_str::<serde_json::Value>(text)
            .is_ok()
            .to_string(),
    )
} }
