use aho_corasick::AhoCorasickBuilder;
use aho_corasick::AhoCorasick;
use std::{
    cell::RefCell,
    collections::hash_map::HashMap
};

struct Replacements {
    pub automaton: AhoCorasick,
    pub replacements: Vec<String>
}

thread_local! {
    static CREPLACE_MAP: RefCell<HashMap<String, Replacements>> = RefCell::new(HashMap::new());
}

byond_fn! { setup_acreplace(key, patternsjson, replacementsjson) {
    let patterns: Vec<String> = serde_json::from_str(patternsjson.clone()).ok()?;
    let replacements: Vec<String> = serde_json::from_str(replacementsjson.clone()).ok()?;
    let ac = AhoCorasickBuilder::new().auto_configure(&patterns).build(&patterns);
    CREPLACE_MAP.with(|cell| {
        let mut map = cell.borrow_mut();
        map.insert(key.to_owned(), Replacements { automaton: ac, replacements: replacements });
    });
    Some("")
} }


byond_fn! { acreplace(key, text) {
    CREPLACE_MAP.with(|cell| -> Option<String> {
        let map = cell.borrow_mut();
        match map.get(&key.to_owned()) {
            Some(replacements) => Some(replacements.automaton.replace_all(text, &replacements.replacements)),
            None => None
        }
    })
} }

