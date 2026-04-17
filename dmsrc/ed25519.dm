/// Generates 32-byte Ed25519 signing key, encoded as standard base64.
/// Returns raw key bytes. Not PEM or PKCS8.
#define rustg_ed25519_generate_secret_key(...) RUSTG_CALL(RUST_G, "ed25519_generate_secret_key")()

/// Derives Ed25519 public key from base64-encoded 32-byte `secret_key`.
/// Returns base64-encoded raw 32-byte public key, or `ERROR: ...`.
#define rustg_ed25519_derive_public_key(secret_key) RUSTG_CALL(RUST_G, "ed25519_derive_public_key")(secret_key)

/// Signs `message` bytes with `secret_key`.
/// Returns base64-encoded raw 64-byte signature, or `ERROR: ...`.
#define rustg_ed25519_sign(secret_key, message) RUSTG_CALL(RUST_G, "ed25519_sign")(secret_key, message)

/// Verifies Ed25519 `signature` over `message` bytes against `public_key`.
/// Returns `"true"`, `"false"`, or `ERROR: ...`.
#define rustg_ed25519_verify(public_key, message, signature) RUSTG_CALL(RUST_G, "ed25519_verify")(public_key, message, signature)
