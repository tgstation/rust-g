use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind, StartKind};
use serde::Deserialize;
use std::{cell::RefCell, collections::hash_map::HashMap};

struct Replacements {
    pub automaton: AhoCorasick,
    pub replacements: Vec<String>,
}

#[derive(Deserialize)]
struct AhoCorasickOptions {
    #[serde(default, deserialize_with = "deserialize_byond_bool")]
    pub anchored: bool,
    #[serde(default, deserialize_with = "deserialize_byond_bool")]
    pub ascii_case_insensitive: bool,
    #[serde(default, deserialize_with = "deserialize_matchkind")]
    pub match_kind: MatchKind,
}

impl AhoCorasickOptions {
    fn auto_configure_and_build(&self, patterns: &[String]) -> AhoCorasick {
        AhoCorasickBuilder::new()
            .start_kind(if self.anchored {
                StartKind::Anchored
            } else {
                StartKind::Unanchored
            })
            .ascii_case_insensitive(self.ascii_case_insensitive)
            .match_kind(self.match_kind)
            .build(patterns)
            .unwrap_or(AhoCorasickBuilder::new().build(patterns).unwrap())
    }
}

fn deserialize_byond_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        _ => Ok(true),
    }
}

fn deserialize_matchkind<'de, D>(deserializer: D) -> Result<MatchKind, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    match String::deserialize(deserializer)?.as_ref() {
        "LeftmostFirst" => Ok(MatchKind::LeftmostFirst),
        "LeftmostLongest" => Ok(MatchKind::LeftmostLongest),
        _ => Ok(MatchKind::Standard),
    }
}

thread_local! {
    static CREPLACE_MAP: RefCell<HashMap<String, Replacements>> = RefCell::new(HashMap::new());
}

byond_fn!(fn setup_acreplace(key, patterns_json, replacements_json) {
    let patterns: Vec<String> = serde_json::from_str(patterns_json).ok()?;
    let replacements: Vec<String> = serde_json::from_str(replacements_json).ok()?;
    let ac = AhoCorasickBuilder::new().build(patterns).unwrap(); // Recommends to just unwrap in the docs
    CREPLACE_MAP.with(|cell| {
        let mut map = cell.borrow_mut();
        map.insert(key.to_owned(), Replacements { automaton: ac, replacements });
    });
    Some("")
});

byond_fn!(fn setup_acreplace_with_options(key, options_json, patterns_json, replacements_json) {
    let options: AhoCorasickOptions = serde_json::from_str(options_json).ok()?;
    let patterns: Vec<String> = serde_json::from_str(patterns_json).ok()?;
    let replacements: Vec<String> = serde_json::from_str(replacements_json).ok()?;
    let ac = options.auto_configure_and_build(&patterns);
    CREPLACE_MAP.with(|cell| {
        let mut map = cell.borrow_mut();
        map.insert(key.to_owned(), Replacements { automaton: ac, replacements });
    });
    Some("")
});

byond_fn!(fn acreplace(key, text) {
    CREPLACE_MAP.with(|cell| -> Option<String> {
        let map = cell.borrow_mut();
        let replacements = map.get(key)?;
        Some(replacements.automaton.replace_all(text, &replacements.replacements))
    })
});

byond_fn!(fn acreplace_with_replacements(key, text, replacements_json) {
    let call_replacements: Vec<String> = serde_json::from_str(replacements_json).ok()?;
    CREPLACE_MAP.with(|cell| -> Option<String> {
        let map = cell.borrow_mut();
        let replacements = map.get(key)?;
        Some(replacements.automaton.replace_all(text, &call_replacements))
    })
});
