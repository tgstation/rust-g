#define rustg_log_write(fname, text, format) call(RUST_G, "log_write")(fname, text, format)
/proc/rustg_log_close_all() return call(RUST_G, "log_close_all")()
