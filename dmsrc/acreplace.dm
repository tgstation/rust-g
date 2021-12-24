#define rustg_setup_acreplace(text, patterns, replacements) call(RUST_G, "setup_acreplace")(text, json_encode(patterns), json_encode(replacements))
#define rustg_acreplace(key, text) call(RUST_G, "acreplace")(key, text)
