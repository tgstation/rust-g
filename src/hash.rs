use crate::error::{Error, Result};
use base64::Engine;
use const_random::const_random;
const XXHASH_SEED: u64 = const_random!(u64);
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use std::{
    cell::RefCell,
    convert::TryInto,
    fs::File,
    hash::Hasher,
    io::Read,
    time::{SystemTime, UNIX_EPOCH},
};
use twox_hash::XxHash64;

byond_fn!(fn hash_string(algorithm, string) {
    string_hash(algorithm, string).ok()
});

byond_fn!(fn decode_base64(string) {
    Some(base64::prelude::BASE64_STANDARD.decode(string).unwrap())
});

byond_fn!(fn hash_file(algorithm, string) {
    file_hash(algorithm, string).ok()
});

byond_fn!(fn generate_totp(hex_seed) {
    match totp_generate(hex_seed, 0, None) {
        Ok(value) => Some(value),
        Err(error) => Some(format!("ERROR: {error:?}"))
    }
});

byond_fn!(fn generate_totp_tolerance(hex_seed, tolerance) {
    let tolerance_value: i32 = match tolerance.parse() {
        Ok(value) => value,
        Err(_) => return Some(String::from("ERROR: Tolerance not a valid integer"))
    };
    match totp_generate_tolerance(hex_seed, tolerance_value, None) {
        Ok(value) => Some(value),
        Err(error) => Some(format!("ERROR: {error:?}"))
    }
});

pub fn string_hash(algorithm: &str, string: &str) -> Result<String> {
    let mut hasher = HashDispatcher::new(algorithm)?;
    hasher.update(string);
    Ok(hasher.finish())
}

const BUFFER_SIZE: usize = 65536;
// don't allocate another buffer every time we hash a file, just reuse the same buffer.
thread_local!( static FILE_HASH_BUFFER: RefCell<[u8; BUFFER_SIZE]> = const { RefCell::new([0; BUFFER_SIZE]) } );

pub fn file_hash(algorithm: &str, path: &str) -> Result<String> {
    let mut hasher = HashDispatcher::new(algorithm)?;
    let mut file = File::open(path)?;

    FILE_HASH_BUFFER.with_borrow_mut(|buffer| {
        loop {
            let bytes_read = file.read(buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        Ok(hasher.finish())
    })
}

/// Generates multiple TOTP codes from 20 character hex_seed, with time step +-tolerance
/// time_override is used as the current unix time instead of the current system time for testing
fn totp_generate_tolerance(
    hex_seed: &str,
    tolerance: i32,
    time_override: Option<i64>,
) -> Result<String> {
    let mut results: Vec<String> = Vec::new();
    for i in -tolerance..(tolerance + 1) {
        let result = totp_generate(hex_seed, i.into(), time_override)?;
        results.push(result)
    }
    Ok(serde_json::to_string(&results)?)
}

/// Generates a single TOTP code from 20 character hex_seed, offset by offset time steps
/// time_override is used as the current unix time instead of the current system time for testing
/// TOTP algorithm described https://blogs.unimelb.edu.au/sciencecommunication/2021/09/30/totp/
/// HMAC algorithm described https://csrc.nist.gov/csrc/media/publications/fips/198/1/final/documents/fips-198-1_final.pdf
fn totp_generate(hex_seed: &str, offset: i64, time_override: Option<i64>) -> Result<String> {
    let mut seed: [u8; 64] = [0; 64];

    match hex::decode_to_slice(hex_seed, &mut seed[..10] as &mut [u8]) {
        Ok(value) => value,
        Err(_) => return Err(Error::HexDecode),
    };

    let ipad: [u8; 64] = seed.map(|x| x ^ 0x36); // HMAC Step 4
    let opad: [u8; 64] = seed.map(|x| x ^ 0x5C); // HMAC Step 7

    // Will panic if the date is not between Jan 1 1970 and the year ~200 billion
    let curr_time: i64 = time_override.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime is before Unix Epoc")
            .as_secs()
            .try_into()
            .unwrap()
    }) / 30;
    let time: u64 = (curr_time + offset) as u64;

    let time_bytes: [u8; 8] = time.to_be_bytes();

    // HMAC Step 5 and 6
    let mut hasher = Sha1::new();
    hasher.update(ipad);
    hasher.update(time_bytes);
    let ipad_time_hash = hasher.finalize();

    // HMAC Step 8 and 9
    hasher = Sha1::new();
    hasher.update(opad);
    hasher.update(ipad_time_hash);
    let hmac = hasher.finalize();

    let offset: usize = (hmac[19] & 0x0F).into();

    let result_bytes: [u8; 4] = hmac[offset..(offset + 4)].try_into().unwrap();

    let full_result: u32 = u32::from_be_bytes(result_bytes);
    let result: u32 = (full_result & 0x7FFFFFFF) % 1000000;

    Ok(result.to_string())
}

enum HashDispatcher {
    Md5(Md5),
    Sha1(Sha1),
    Sha256(Sha256),
    Sha512(Sha512),
    Xxh64(XxHash64),
    Base64(Vec<u8>),
}

impl HashDispatcher {
    fn new(name: &str) -> Result<Self> {
        match name {
            "md5" => Ok(Self::Md5(Md5::new())),
            "sha1" => Ok(Self::Sha1(Sha1::new())),
            "sha256" => Ok(Self::Sha256(Sha256::new())),
            "sha512" => Ok(Self::Sha512(Sha512::new())),
            "xxh64" => Ok(Self::Xxh64(XxHash64::with_seed(XXHASH_SEED))),
            "xxh64_fixed" => Ok(Self::Xxh64(XxHash64::with_seed(17479268743136991876))),
            "base64" => Ok(Self::Base64(Vec::new())),
            _ => Err(Error::InvalidAlgorithm),
        }
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        let data = data.as_ref();
        match self {
            HashDispatcher::Md5(hasher) => hasher.update(data),
            HashDispatcher::Sha1(hasher) => hasher.update(data),
            HashDispatcher::Sha256(hasher) => hasher.update(data),
            HashDispatcher::Sha512(hasher) => hasher.update(data),
            HashDispatcher::Xxh64(hasher) => hasher.write(data),
            HashDispatcher::Base64(buffer) => buffer.extend_from_slice(data),
        }
    }

    fn finish(self) -> String {
        match self {
            HashDispatcher::Md5(hasher) => hex::encode(hasher.finalize()),
            HashDispatcher::Sha1(hasher) => hex::encode(hasher.finalize()),
            HashDispatcher::Sha256(hasher) => hex::encode(hasher.finalize()),
            HashDispatcher::Sha512(hasher) => hex::encode(hasher.finalize()),
            HashDispatcher::Xxh64(hasher) => format!("{:x}", hasher.finish()),
            HashDispatcher::Base64(buffer) => base64::prelude::BASE64_STANDARD.encode(&buffer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totp_generate_test() {
        // The big offset is so that it always uses the same time, allowing for verification that the algorithm is correct
        // Seed, time, and result for zero offset taken from https://blogs.unimelb.edu.au/sciencecommunication/2021/09/30/totp/
        let result = totp_generate("B93F9893199AEF85739C", 0, Some(54424722i64 * 30 + 29));
        assert_eq!(result.unwrap(), "417714");
        let result2 = totp_generate("B93F9893199AEF85739C", -1, Some(54424722i64 * 30 + 29));
        assert_eq!(result2.unwrap(), "358747");
        let result3 = totp_generate("B93F9893199AEF85739C", 1, Some(54424722i64 * 30 + 29));
        assert_eq!(result3.unwrap(), "539257");
        let result4 = totp_generate("B93F9893199AEF85739C", 2, Some(54424722i64 * 30 + 29));
        assert_eq!(result4.unwrap(), "679828");
        let json_result =
            totp_generate_tolerance("B93F9893199AEF85739C", 1, Some(54424722i64 * 30 + 29));
        assert_eq!(json_result.unwrap(), "[\"358747\",\"417714\",\"539257\"]");
        let err_result = totp_generate_tolerance("66", 0, None);
        assert!(err_result.is_err());
    }
}
