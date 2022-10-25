#define rustg_file_read(fname) RGCALL(RUST_G, "file_read")(fname)
#define rustg_file_exists(fname) RGCALL(RUST_G, "file_exists")(fname)
#define rustg_file_write(text, fname) RGCALL(RUST_G, "file_write")(text, fname)
#define rustg_file_append(text, fname) RGCALL(RUST_G, "file_append")(text, fname)
#define rustg_file_get_line_count(fname) text2num(RGCALL(RUST_G, "file_get_line_count")(fname))
#define rustg_file_seek_line(fname, line) RGCALL(RUST_G, "file_seek_line")(fname, "[line]")

#ifdef RUSTG_OVERRIDE_BUILTINS
	#define file2text(fname) rustg_file_read("[fname]")
	#define text2file(text, fname) rustg_file_append(text, "[fname]")
#endif
