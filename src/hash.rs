use crate::error::{Error, Result};
use const_random::const_random;
const XXHASH_SEED: u64 = const_random!(u64);
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use std::{
    fs::File,
    hash::Hasher,
    io::{BufReader, Read},
    time::{SystemTime, UNIX_EPOCH},
    convert::TryInto,
};
use twox_hash::XxHash64;

byond_fn! { hash_string(algorithm, string) {
    string_hash(algorithm, string).ok()
} }

byond_fn! { hash_file(algorithm, string) {
    file_hash(algorithm, string).ok()
} }

byond_fn! { generate_totp(hex_seed) {
    totp_generate(hex_seed).ok()
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
            Ok(format!("{:x}", hasher.finish()))
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

fn totp_generate(hex_seed: &str) -> Result<String> {
    let mut seed: [u8; 64] = [0; 64];

    let hex_seed_bytes = (0..hex_seed.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&hex_seed[i..i + 2], 16));

    let mut i = 0;
    for b in hex_seed_bytes {
        seed[i] = b.unwrap();
        i += 1;
    }

    let mut ipad: [u8; 64] = [0; 64];
    let mut opad: [u8; 64] = [0; 64];

    for j in 0..64 {
        ipad[j] = seed[j] ^ 0x36;
        opad[j] = seed[j] ^ 0x5C;
    }
    let time: u64 = SystemTime::now().duration_since(UNIX_EPOCH).expect("SystemTime before UNIX EPOC").as_secs() / 30;

    let time_bytes: [u8; 8] = time.to_be_bytes();

    let mut ipad_time: [u8; 72] = [0; 72];

    for j in 0..72 {
        if j < 64 {
            ipad_time[j] = ipad[j];
        } else {
            ipad_time[j] = time_bytes[j - 64];
        }
    }

    let mut hasher = Sha1::new();
    hasher.update(ipad_time);
    let ipad_time_hash = hasher.finalize();

    let mut ipad_time_hash_opad: [u8; 84] = [0; 84];

    for j in 0..84 {
        if j < 64 {
            ipad_time_hash_opad[j] = opad[j]
        } else {
            ipad_time_hash_opad[j] = ipad_time_hash[j - 64]
        }
    }

    hasher = Sha1::new();
    hasher.update(ipad_time_hash_opad);
    let hmac = hasher.finalize();

    let offset: usize = (hmac[19] & 0x0F).into();

    let result_bytes: [u8; 4] = hmac[offset..(offset+4)].try_into().expect("wrong length");

    let full_result: u32 = u32::from_be_bytes(result_bytes);
    let result: u32 = (full_result & 0x7FFFFFFF) % 1000000;

    Ok(result.to_string())
}
