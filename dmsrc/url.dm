#define rustg_url_encode(text) RGCALL(RUST_G, "url_encode")("[text]")
#define rustg_url_decode(text) RGCALL(RUST_G, "url_decode")(text)

#ifdef RUSTG_OVERRIDE_BUILTINS
	#define url_encode(text) rustg_url_encode(text)
	#define url_decode(text) rustg_url_decode(text)
#endif
