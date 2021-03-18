use crate::error::{Error, Result};
use const_random::const_random  ;
const XXHASH_SEED: u64 = const_random!(u64);
use twox_hash::XxHash64;
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use std::{
    fs::File,
    io::{BufReader, Read},
    hash::Hasher,
};



byond_fn! { hash_string(algorithm, string) {
    string_hash(algorithm, string).ok()
} }

byond_fn! { hash_file(algorithm, string) {
    file_hash(algorithm, string).ok()
} }

fn hash_algorithm<B: AsRef<[u8]>>(name: &str, bytes: B) -> Result<String> {
    match name {
        "md5" => {
            let mut hasher = Md5::new();
            hasher.update(bytes.as_ref());
            Ok(hex::encode(hasher.finalize()))
        }
        "sha1" => {
            let mut hasher = Sha1::new();
            hasher.update(bytes.as_ref());
            Ok(hex::encode(hasher.finalize()))
        }
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(bytes.as_ref());
            Ok(hex::encode(hasher.finalize()))
        }
        "sha512" => {
            let mut hasher = Sha512::new();
            hasher.update(bytes.as_ref());
            Ok(hex::encode(hasher.finalize()))
        }
        "xxh64" => {
            let mut hasher = XxHash64::with_seed(XXHASH_SEED);
            hasher.write(bytes.as_ref());
            Ok(format!("{:x}",hasher.finish()))
        }
        "base64" => {
            Ok(base64::encode(bytes.as_ref()))
        }
        _ => Err(Error::InvalidAlgorithm),
    }
}

fn string_hash(algorithm: &str, string: &str) -> Result<String> {
    Ok(hash_algorithm(algorithm, string)?)
}

fn file_hash(algorithm: &str, path: &str) -> Result<String> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut file = BufReader::new(File::open(path)?);
    file.read_to_end(&mut bytes)?;

    Ok(hash_algorithm(algorithm, &bytes)?)
}
