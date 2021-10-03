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
    totp_generate(hex_seed, 0, None).ok()
} }

byond_fn! { generate_totp_tolerance(hex_seed, tolerance) {
    totp_generate_tolerance(hex_seed, tolerance.parse().unwrap(), None).ok()
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

fn totp_generate_tolerance(hex_seed: &str, tolerance: i32, time_override: Option<i64>) -> Result<String> {
    let mut results: Vec<String> = Vec::new();
    for i in -tolerance..(tolerance + 1) {
        results.push(totp_generate(hex_seed, i.try_into().unwrap(), time_override).unwrap())
    }
    Ok(serde_json::to_string(&results).unwrap())
}

fn totp_generate(hex_seed: &str, offset: i64, time_override: Option<i64>) -> Result<String> {

    let mut seed: [u8; 64] = [0; 64];

    hex::decode_to_slice(hex_seed, &mut seed[..10] as &mut [u8]).unwrap();

    let ipad: [u8; 64] = seed.map(|x| x ^ 0x36);
    let opad: [u8; 64] = seed.map(|x| x ^ 0x5C);

    let curr_time: i64 = time_override.unwrap_or(SystemTime::now().duration_since(UNIX_EPOCH).expect("SystemTime before UNIX EPOC").as_secs().try_into().unwrap()) / 30;
    let time: u64 = (curr_time + offset) as u64;

    let time_bytes: [u8; 8] = time.to_be_bytes();

    let mut hasher = Sha1::new();
    hasher.update(ipad);
    hasher.update(time_bytes);
    let ipad_time_hash = hasher.finalize();

    hasher = Sha1::new();
    hasher.update(opad);
    hasher.update(ipad_time_hash);
    let hmac = hasher.finalize();

    let offset: usize = (hmac[19] & 0x0F).into();

    let result_bytes: [u8; 4] = hmac[offset..(offset+4)].try_into().expect("wrong length");

    let full_result: u32 = u32::from_be_bytes(result_bytes);
    let result: u32 = (full_result & 0x7FFFFFFF) % 1000000;

    Ok(result.to_string())
}

#[cfg(feature = "hash")]
#[test]
fn totp_generate_test() {
    // The big offset is so that it always uses the same time, allowing for verification that the algorithm is correct
    // Token, time, and result for zero offset taken from https://blogs.unimelb.edu.au/sciencecommunication/2021/09/30/totp/
    let result = totp_generate("B93F9893199AEF85739C", 0, Some(54424722i64 * 30 + 29));
    assert_eq!(result.unwrap(), "417714");
    let result2 = totp_generate("B93F9893199AEF85739C", -1, Some(54424722i64 * 30 + 29));
    assert_eq!(result2.unwrap(), "358747");
    let result3 = totp_generate("B93F9893199AEF85739C", 1, Some(54424722i64 * 30 + 29));
    assert_eq!(result3.unwrap(), "539257");
    let result4 = totp_generate("B93F9893199AEF85739C", 2, Some(54424722i64 * 30 + 29));
    assert_eq!(result4.unwrap(), "679828");
    let json_result = totp_generate_tolerance("B93F9893199AEF85739C", 1, Some(54424722i64 * 30 + 29));
    assert_eq!(json_result.unwrap(), "[\"358747\",\"417714\",\"539257\"]");
}
