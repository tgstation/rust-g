//! Buildscript which will save a `rust_g.dm` with the DLL's public API.

use std::io::Write;
use std::fs::File;

fn main() {
    let mut f = File::create("target/rust_g.dm").unwrap();

    // header
    write!(f, r#"// rust_g.dm - DM API for rust_g extension library
#define RUST_G "rust_g"
"#).unwrap();

    // module: dmi
    write!(f, r#"
#define rustg_dmi_strip_metadata(fname) call(RUST_G, "dmi_strip_metadata")(fname)
"#).unwrap();

    // module: file
    write!(f, r#"
#define rustg_file_read(fname) call(RUST_G, "file_read")(fname)
#define rustg_file_write(text, fname) call(RUST_G, "file_write")(text, fname)

#ifdef RUSTG_OVERRIDE_BUILTINS
#define file2text(fname) rustg_file_read(fname)
#define text2file(text, fname) rustg_file_write(text, fname)
#endif
"#).unwrap();

    // module: hash
    write!(f, r#"
#define rustg_hash_string(algorithm, text) call(RUST_G, "hash_string")(algorithm, text)
#define rustg_hash_file(algorithm, fname) call(RUST_G, "hash_file")(algorithm, fname)

#define RUSTG_HASH_MD5 "md5"
#define RUSTG_HASH_SHA1 "sha1"
#define RUSTG_HASH_SHA256 "sha256"
#define RUSTG_HASH_SHA512 "sha512"

#ifdef RUSTG_OVERRIDE_BUILTINS
#define md5(thing) (isfile(thing) ? rustg_hash_file(RUSTG_HASH_MD5, "[thing]") : rustg_hash_string(RUSTG_HASH_MD5, thing))
#endif
"#).unwrap();

    // module: log
    write!(f, r#"
#define rustg_log_write(fname, text) call(RUST_G, "log_write")(fname, text)
#define rustg_log_close_all() call(RUST_G, "log_close_all")()
"#).unwrap();

    // module: url
    write!(f, r#"
#define rustg_url_encode(text) call(RUST_G, "url_encode")(text)
#define rustg_url_decode(text) call(RUST_G, "url_decode")(text)

#ifdef RUSTG_OVERRIDE_BUILTINS
#define url_encode(text) rustg_url_encode(text)
#define url_decode(text) rustg_url_decode(text)
#endif
"#).unwrap();
}
