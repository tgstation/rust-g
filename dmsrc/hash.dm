
#define rustg_hash_string(algorithm, text) call(RUST_G, "hash_string")(algorithm, text)
#define rustg_hash_file(algorithm, fname) call(RUST_G, "hash_file")(algorithm, fname)
#define rustg_hash_generate_totp(seed) call(RUST_G, "generate_totp")(seed)
#define rustg_hash_generate_totp_tolerance(seed, tolerance) call(RUST_G, "generate_totp_tolerance")(seed, tolerance)

#define RUSTG_HASH_MD5 "md5"
#define RUSTG_HASH_SHA1 "sha1"
#define RUSTG_HASH_SHA256 "sha256"
#define RUSTG_HASH_SHA512 "sha512"
#define RUSTG_HASH_XXH64 "xxh64"
#define RUSTG_HASH_BASE64 "base64"

#ifdef RUSTG_OVERRIDE_BUILTINS
	#define md5(thing) (isfile(thing) ? rustg_hash_file(RUSTG_HASH_MD5, "[thing]") : rustg_hash_string(RUSTG_HASH_MD5, thing))
#endif
