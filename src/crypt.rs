use jobs;
use bcrypt::{hash, verify, DEFAULT_COST};
use ed25519_dalek::{PublicKey, Signature};

byond_fn! { bcrypt_hash(data) {
    let data = data.to_owned();
    Some(jobs::start(move || {
		hash(data.to_owned(), DEFAULT_COST).unwrap_or("ERRNOHASH".to_string())
	}))
}}

byond_fn! { bcrypt_verify(data, hash) {
    let data = data.to_owned();
	let hash = hash.to_owned();
	Some(jobs::start(move || {
		if verify(data, &hash).unwrap_or(false) { "yes".to_string() } else { "no".to_string() }
	}))
}}

byond_fn! { ed25519_verify(data, sig, pubkey) {
    Some(match safe_ed25519_verify(data, sig, pubkey) {
        Ok(_) => "yes",
        Err(e) => e
    })
}}

fn safe_ed25519_verify(data: &str, sig: &str, pubkey: &str) -> Result<(), &'static str> {
    let pubdata = hex::decode(&pubkey as &str).map_err(|_| "invalid key hex")?;
    let sigdata = hex::decode(&sig as &str).map_err(|_| "invalid sig hex")?;
    let datadata = hex::decode(&data as &str).map_err(|_| "invalid data hex")?;
    let key = PublicKey::from_bytes(&pubdata).map_err(|_| "invalid key")?;
    let sig = Signature::from_bytes(&sigdata).map_err(|_| "invalid sig")?;
    if key.verify(&datadata, &sig).is_ok() { return Ok(()); } else { return Err("no"); };
}