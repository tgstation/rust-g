#define rustg_log_write(fname, text, format) RGCALL(RUST_G, "log_write")(fname, text, format)
/proc/rustg_log_close_all() return RGCALL(RUST_G, "log_close_all")()
