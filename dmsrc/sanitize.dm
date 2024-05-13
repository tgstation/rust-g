/*
 * Takes in a string and json_encode()"d lists to produce a sanitized string.
 * This function operates on whitelists, there is currently no way to blacklist.
 * Args:
 * * text: the string to sanitize.
 * * attribute_whitelist_json: a json_encode()'d list of HTML attributes to allow in the final string.
 * * tag_whitelist_json: a json_encode()'d list of HTML tags to allow in the final string.
 */
#define rustg_sanitize_html(text, attribute_whitelist_json, tag_whitelist_json) RUSTG_CALL(RUST_G, "sanitize_html")(text, attribute_whitelist_json, tag_whitelist_json)
