use crate::error::Result;
use std::borrow::Cow;
use url_dep::form_urlencoded::byte_serialize;

byond_fn!(fn url_encode(data) {
    Some(encode(data))
});

byond_fn!(fn url_decode(data) {
    decode(data).ok()
});

fn encode(string: &str) -> String {
    byte_serialize(string.as_bytes()).collect()
}

fn decode(string: &str) -> Result<String> {
    let replaced = replace_plus(string.as_bytes());
    // into_owned() is not strictly necessary here, but saves some refactoring work.
    Ok(percent_encoding::percent_decode(&replaced)
        .decode_utf8_lossy()
        .into_owned())
}

// From `url` crate.
/// Replace b'+' with b' '
fn replace_plus<'a>(input: &'a [u8]) -> Cow<'a, [u8]> {
    match input.iter().position(|&b| b == b'+') {
        None => Cow::Borrowed(input),
        Some(first_position) => {
            let mut replaced = input.to_owned();
            replaced[first_position] = b' ';
            for byte in &mut replaced[first_position + 1..] {
                if *byte == b'+' {
                    *byte = b' ';
                }
            }
            Cow::Owned(replaced)
        }
    }
}
