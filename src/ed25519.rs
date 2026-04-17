use crate::error::{Error, Result};
use base64::{Engine, prelude::BASE64_STANDARD};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use zeroize::Zeroize;

const ED25519_SECRET_KEY_LEN: usize = 32;
const ED25519_PUBLIC_KEY_LEN: usize = 32;
const ED25519_SIGNATURE_LEN: usize = 64;

byond_fn!(
    fn ed25519_generate_secret_key() {
        Some(generate_ed25519_secret_key())
    }
);

byond_fn!(fn ed25519_derive_public_key(secret_key) {
    match derive_ed25519_public_key(secret_key) {
        Ok(public_key) => Some(public_key),
        Err(error) => Some(format!("ERROR: {error}")),
    }
});

byond_fn!(fn ed25519_sign(secret_key, message) {
    match sign_ed25519(secret_key, message.as_bytes()) {
        Ok(signature) => Some(signature),
        Err(error) => Some(format!("ERROR: {error}")),
    }
});

byond_fn!(fn ed25519_verify(public_key, message, signature) {
    match verify_ed25519(public_key, message.as_bytes(), signature) {
        Ok(is_valid) => Some(is_valid.to_string()),
        Err(error) => Some(format!("ERROR: {error}")),
    }
});

pub fn generate_ed25519_secret_key() -> String {
    let mut secret_key = [0u8; ED25519_SECRET_KEY_LEN];
    rand::fill(&mut secret_key[..]);
    let encoded = BASE64_STANDARD.encode(secret_key);
    secret_key.zeroize();
    encoded
}

pub fn derive_ed25519_public_key(secret_key_b64: &str) -> Result<String> {
    let signing_key = decode_ed25519_signing_key(secret_key_b64)?;
    Ok(BASE64_STANDARD.encode(signing_key.verifying_key().to_bytes()))
}

pub fn sign_ed25519(secret_key_b64: &str, message: &[u8]) -> Result<String> {
    let signing_key = decode_ed25519_signing_key(secret_key_b64)?;
    let signature = signing_key.sign(message);
    Ok(BASE64_STANDARD.encode(signature.to_bytes()))
}

pub fn verify_ed25519(public_key_b64: &str, message: &[u8], signature_b64: &str) -> Result<bool> {
    let verifying_key = decode_ed25519_verifying_key(public_key_b64)?;
    let signature = decode_ed25519_signature(signature_b64)?;
    Ok(verifying_key.verify_strict(message, &signature).is_ok())
}

fn decode_ed25519_signing_key(secret_key_b64: &str) -> Result<SigningKey> {
    let mut decoded = BASE64_STANDARD.decode(secret_key_b64)?;
    let actual = decoded.len();
    if actual != ED25519_SECRET_KEY_LEN {
        decoded.zeroize();
        return Err(Error::InvalidEd25519Length {
            kind: "secret key",
            expected: ED25519_SECRET_KEY_LEN,
            actual,
        });
    }

    let mut secret_key = [0u8; ED25519_SECRET_KEY_LEN];
    secret_key.copy_from_slice(&decoded);
    decoded.zeroize();

    let signing_key = SigningKey::from_bytes(&secret_key);
    secret_key.zeroize();
    Ok(signing_key)
}

fn decode_ed25519_verifying_key(public_key_b64: &str) -> Result<VerifyingKey> {
    let decoded = BASE64_STANDARD.decode(public_key_b64)?;
    let public_key: [u8; ED25519_PUBLIC_KEY_LEN] =
        decoded
            .try_into()
            .map_err(|decoded: Vec<u8>| Error::InvalidEd25519Length {
                kind: "public key",
                expected: ED25519_PUBLIC_KEY_LEN,
                actual: decoded.len(),
            })?;
    Ok(VerifyingKey::from_bytes(&public_key)?)
}

fn decode_ed25519_signature(signature_b64: &str) -> Result<Signature> {
    let decoded = BASE64_STANDARD.decode(signature_b64)?;
    if decoded.len() != ED25519_SIGNATURE_LEN {
        return Err(Error::InvalidEd25519Length {
            kind: "signature",
            expected: ED25519_SIGNATURE_LEN,
            actual: decoded.len(),
        });
    }
    Ok(Signature::from_slice(&decoded)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ed25519_round_trip() {
        let secret_key = generate_ed25519_secret_key();
        let public_key = derive_ed25519_public_key(&secret_key).unwrap();
        let message = b"rust-g ed25519 test";
        let signature = sign_ed25519(&secret_key, message).unwrap();

        assert!(verify_ed25519(&public_key, message, &signature).unwrap());
        assert!(!verify_ed25519(&public_key, b"rust-g ed25519 toast", &signature).unwrap());
    }

    #[test]
    fn ed25519_matches_rfc8032() {
        // https://datatracker.ietf.org/doc/html/rfc8032#section-7.1
        let secret_key1 = BASE64_STANDARD.encode(decode_hex(
            "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60",
        ));
        let public_key1 = BASE64_STANDARD.encode(decode_hex(
            "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
        ));
        let signature1 = BASE64_STANDARD.encode(decode_hex(
            "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b",
        ));

        assert_eq!(
            derive_ed25519_public_key(&secret_key1).unwrap(),
            public_key1
        );
        assert_eq!(sign_ed25519(&secret_key1, b"").unwrap(), signature1);
        assert!(verify_ed25519(&public_key1, b"", &signature1).unwrap());

        let secret_key2 = BASE64_STANDARD.encode(decode_hex(
            "4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb",
        ));
        let public_key2 = BASE64_STANDARD.encode(decode_hex(
            "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
        ));
        let signature2 = BASE64_STANDARD.encode(decode_hex(
            "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00",
        ));

        assert_eq!(
            derive_ed25519_public_key(&secret_key2).unwrap(),
            public_key2
        );
        assert_eq!(sign_ed25519(&secret_key2, b"r").unwrap(), signature2);
        assert!(verify_ed25519(&public_key2, b"r", &signature2).unwrap());
    }

    #[test]
    fn invalid_secret_key_length_errors() {
        let error = derive_ed25519_public_key(&BASE64_STANDARD.encode([0u8; 31])).unwrap_err();
        assert!(matches!(
            error,
            Error::InvalidEd25519Length {
                kind: "secret key",
                expected: ED25519_SECRET_KEY_LEN,
                actual: 31,
            }
        ));
    }

    // I don't really want to load the hex dep
    fn decode_hex(input: &str) -> Vec<u8> {
        input
            .as_bytes()
            .chunks_exact(2)
            .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
            .collect()
    }
}
