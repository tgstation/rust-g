use regex::Regex;
use serde::Serialize;

byond_fn!(
    fn regex_captures(pattern, text, start) {
        let start: usize = match start.parse() {
            Ok(start) => start,
            Err(_) => {
                return serde_json::to_string(&serde_json::json!({
                    "success": false,
                    "reason": "invalid start index",
                })).ok();
            }
        };

        serde_json::to_string(&match regex_captures_impl(pattern, text, start) {
            Ok(captures) => serde_json::json!({
                "success": true,
                "result": captures,
            }),

            Err(error) => serde_json::json!({
                "success": false,
                "reason": error.to_string(),
            }),
        }).ok()
    }
);

#[derive(Serialize)]
struct CaptureResult {
    captures: Vec<String>,

    index: usize,
    next: usize,

    #[serde(alias = "match")]
    the_match: String,
}

fn regex_captures_impl(
    pattern: &str,
    text: &str,
    start: usize,
) -> Result<Option<CaptureResult>, regex::Error> {
    let regex = Regex::new(pattern)?;

    let mut locations = regex.capture_locations();
    let the_match = match regex.captures_read_at(&mut locations, text, start) {
        Some(captures) => captures,
        None => return Ok(None),
    };

    let mut captures = Vec::with_capacity(locations.len().saturating_sub(1));

    for i in 1..locations.len() {
        let (start, end) = locations.get(i).expect("invalid capture location");
        captures.push(text[start..end].to_string());
    }

    Ok(Some(CaptureResult {
        captures,

        index: the_match.start(),
        next: the_match.end(),

        the_match: the_match.as_str().to_owned(),
    }))
}
