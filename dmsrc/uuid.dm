/// Generates a version 4 UUID.
/// See https://www.ietf.org/rfc/rfc9562.html#section-5.4 for specifics on version 4 UUIDs.
#define rustg_generate_uuid_v4(...) RUSTG_CALL(RUST_G, "uuid_v4")()

/// Generates a version 7 UUID, with the current time.
/// See https://www.ietf.org/rfc/rfc9562.html#section-5.7 for specifics on version 7 UUIDs.
#define rustg_generate_uuid_v7(...) RUSTG_CALL(RUST_G, "uuid_v7")()

/// Generates a random version 2 CUID.
/// See https://github.com/paralleldrive/cuid2 for specifics on version 2 CUIDs.
#define rustg_generate_cuid2(...) RUSTG_CALL(RUST_G, "cuid2")()

/// Generates a random version 2 CUID with the given length.
/// See https://github.com/paralleldrive/cuid2 for specifics on version 2 CUIDs.
#define rustg_generate_cuid2_length(length) RUSTG_CALL(RUST_G, "cuid2_len")("[length]")
