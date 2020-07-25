use crate::error::{Error, Result};
use url_dep::form_urlencoded::{byte_serialize, parse};

byond_fn! { url_encode(data) {
    Some(encode(data))
} }

byond_fn! { url_decode(data) {
    decode(data).ok()
} }

fn encode(string: &str) -> String {
    byte_serialize(string.as_bytes()).collect()
}

fn decode(string: &str) -> Result<String> {
    let decoded: String = parse(string.as_bytes())
        .map(|(key, val)| [key, val].concat())
        .collect();

    if decoded.contains('\0') {
        return Err(Error::Null);
    }

    Ok(decoded)
}
