use crate::error::Result;
use std::collections::HashSet;
use maplit::hashset;

byond_fn!(fn sanitize_html(text, attribute_whitelist_json, tag_whitelist_json) {
    match seriously_sanitize_html(text, attribute_whitelist_json, tag_whitelist_json) {
        Ok(r) => return Some(r),
        Err(e) => return Some(e.to_string())
    }
});

fn seriously_sanitize_html(text: &str, attribute_whitelist_json: &str, tag_whitelist_json: &str) -> Result<String> {
    let attribute_whitelist: HashSet<&str> = serde_json::from_str(attribute_whitelist_json)?;
    let tag_whitelist: HashSet<&str> = serde_json::from_str(tag_whitelist_json)?;

    let mut prune_url_schemes = ammonia::Builder::default().clone_url_schemes();
    prune_url_schemes.insert("byond");

    let sanitized = ammonia::Builder::empty()
    .clean_content_tags(hashset!["script", "style"]) // Completely forbid script and style attributes.
    .link_rel(Some("noopener")) // https://mathiasbynens.github.io/rel-noopener/
    .url_schemes(prune_url_schemes)
    .generic_attributes(attribute_whitelist)
    .tags(tag_whitelist)
    .clean(text)
    .to_string();


    Ok(sanitized)
}
