use crate::error::{Error, Result};
use crypto_hash::{Algorithm, Hasher};
use std::{fs::File, io};

byond_fn! { hash_string(algorithm, string) {
    string_hash(algorithm, string).ok()
} }

byond_fn! { hash_file(algorithm, string) {
    file_hash(algorithm, string).ok()
} }

fn get_algorithm(string: &str) -> Result<Algorithm> {
    let algorithm = match string {
        "md5" => Algorithm::MD5,
        "sha1" => Algorithm::SHA1,
        "sha256" => Algorithm::SHA256,
        "sha512" => Algorithm::SHA512,
        _ => return Err(Error::InvalidAlgorithm),
    };

    Ok(algorithm)
}

fn string_hash(algorithm: &str, string: &str) -> Result<String> {
    let algorithm = get_algorithm(algorithm)?;
    let digest = crypto_hash::digest(algorithm, string.as_bytes());

    Ok(hex::encode(digest))
}

fn file_hash(algorithm: &str, path: &str) -> Result<String> {
    let algorithm = get_algorithm(algorithm)?;

    let mut file = File::open(path)?;
    let mut digest = Hasher::new(algorithm);

    io::copy(&mut file, &mut digest)?;
    Ok(hex::encode(digest.finish()))
}
