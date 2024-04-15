use crate::error::Result;
use rand::Rng;

byond_fn!(fn random_replacetext(text, prob, rand_char) {
    replacetext(text, prob, rand_char).ok()
});

fn replacetext(text: &str, prob_as_str: &str, replacement_str: &str) -> Result<String> {
    let prob = prob_as_str.parse::<u32>()?;
    let mut rng = rand::thread_rng();
    let mut string_return = String::new();

    for character in text.chars() {
        if character.is_whitespace() {
            string_return.push(character);
            continue;
        }

        if rng.gen_ratio(prob, 100) {
            string_return.push_str(replacement_str);
        } else {
            string_return.push(character);
        }

    }
    Ok(string_return)
}
