#define rustg_hash_string(algorithm, text) RUSTG_CALL(RUST_G, "hash_string")(algorithm, text)
#define rustg_hash_file(algorithm, fname) RUSTG_CALL(RUST_G, "hash_file")(algorithm, fname)
#define rustg_hash_generate_totp(seed) RUSTG_CALL(RUST_G, "generate_totp")(seed)
#define rustg_hash_generate_totp_tolerance(seed, tolerance) RUSTG_CALL(RUST_G, "generate_totp_tolerance")(seed, tolerance)

/// Creates a cryptographically-secure pseudorandom number generator using the OS-level PRNG as a seed
/// n_bytes is the number of bytes provided to the RNG, the length of the string output varies by format
/// The output string length and characters contained in each format is as follows:
/// RUSTG_RNG_FORMAT_HEX: n_bytes * 2, [a-z0-9]
/// RUSTG_RNG_FORMAT_ALPHANUMERIC: n_bytes, [A-Za-z0-9]
/// RUSTG_RNG_FORMAT_BASE64: 4 * ceil(n_bytes/3), [A-Za-z0-9+/=]
/// Outputs "ERROR: [reason]" if the format string provided is invalid, or n_bytes is not a positive non-zero integer
#define rustg_csprng_chacha20(format, n_bytes) RUSTG_CALL(RUST_G, "csprng_chacha20")(format, n_bytes)

/// Creates a seeded pseudorandom number generator using the SHA256 hash output bytes of the seed string
/// Note that this function is NOT suitable for use in cryptography and is intended for high-quality **predictable** RNG
/// Use rustg_csprng_chacha20 for a cryptographically-secure PRNG.
/// n_bytes is the number of bytes provided to the RNG, the length of the string output varies by format
/// The output string length and characters contained in each format is as follows:
/// RUSTG_RNG_FORMAT_HEX: n_bytes * 2, [a-z0-9]
/// RUSTG_RNG_FORMAT_ALPHANUMERIC: n_bytes, [A-Za-z0-9]
/// RUSTG_RNG_FORMAT_BASE64: 4 * ceil(n_bytes/3), [A-Za-z0-9+/=]
/// Outputs "ERROR: [reason]" if the format string provided is invalid, or n_bytes is not a positive non-zero integer
#define rustg_prng_chacha20_seeded(format, n_bytes, seed) RUSTG_CALL(RUST_G, "prng_chacha20_seeded")(format, n_bytes, seed)

#define RUSTG_RNG_FORMAT_HEX "hex"
#define RUSTG_RNG_FORMAT_ALPHANUMERIC "alphanumeric"
#define RUSTG_RNG_FORMAT_BASE64 "base64"

#define RUSTG_HASH_MD5 "md5"
#define RUSTG_HASH_SHA1 "sha1"
#define RUSTG_HASH_SHA256 "sha256"
#define RUSTG_HASH_SHA512 "sha512"
#define RUSTG_HASH_XXH64 "xxh64"
#define RUSTG_HASH_BASE64 "base64"

/// Encode a given string into base64
#define rustg_encode_base64(str) rustg_hash_string(RUSTG_HASH_BASE64, str)
/// Decode a given base64 string
#define rustg_decode_base64(str) RUSTG_CALL(RUST_G, "decode_base64")(str)

#ifdef RUSTG_OVERRIDE_BUILTINS
	#define md5(thing) (isfile(thing) ? rustg_hash_file(RUSTG_HASH_MD5, "[thing]") : rustg_hash_string(RUSTG_HASH_MD5, thing))
#endif
