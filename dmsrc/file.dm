#define rustg_file_read(fname) call(RUST_G, "file_read")(fname)
#define rustg_file_exists(fname) call(RUST_G, "file_exists")(fname)
#define rustg_file_write(text, fname) call(RUST_G, "file_write")(text, fname)
#define rustg_file_append(text, fname) call(RUST_G, "file_append")(text, fname)

#ifdef RUSTG_OVERRIDE_BUILTINS
	#define file2text(fname) rustg_file_read("[fname]")
	#define text2file(text, fname) rustg_file_append(text, "[fname]")
#endif
