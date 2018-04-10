use percent_encoding::{percent_decode, utf8_percent_encode, PATH_SEGMENT_ENCODE_SET};

use error::{Error, Result};

byond_function! { url_encode(data) {
    Some(encode(data))
} }

byond_function! { url_decode(data) {
    decode(data).ok()
} }

fn encode(string: &str) -> String {
    utf8_percent_encode(string, PATH_SEGMENT_ENCODE_SET).to_string()
}

fn decode(string: &str) -> Result<String> {
    let decoded = percent_decode(string.as_bytes())
        .decode_utf8()?
        .into_owned();

    if decoded.contains('\0') {
        return Err(Error::Null);
    }

    Ok(decoded)
}
