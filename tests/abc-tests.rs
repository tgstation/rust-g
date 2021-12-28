extern crate regex;
use regex::{Regex,RegexBuilder,CaptureMatches};
use std::cmp::Ordering;

fn are_captures_sorted(matches: CaptureMatches, context: &str) -> Result<(), String> {
    let mut prev_string = "";
    for cap in matches {
        let capstring = cap.get(0).unwrap().as_str();
        match prev_string.cmp(&capstring) {
            Ordering::Greater => return Err(format!("{} is not sorted in {}", &capstring, &context)),
            _ => { prev_string = capstring; }
         };
    }
    Ok(())
}


#[test]
fn test_readme() -> Result<(), String> {
    let readme = std::fs::read_to_string("README.md").unwrap();
    let blocksre = RegexBuilder::new(r"^The default features are:\r?\n((:?^.+?\r?\n)*)\r?\nAdditional features are:\r?\n((:?^.+?\r?\n)*)").multi_line(true).build().unwrap();
    let linesre = RegexBuilder::new(r"^\*(.+?)$").multi_line(true).build().unwrap();
    let blocks = blocksre.captures(&readme).unwrap();
    are_captures_sorted(linesre.captures_iter(blocks.get(1).unwrap().as_str()), "README.md default features")?;
    are_captures_sorted(linesre.captures_iter(blocks.get(3).unwrap().as_str()), "README.md additional features")
}

#[test]
fn test_librs() -> Result<(), String> {
    let librs = std::fs::read_to_string("src/lib.rs").unwrap();
    let modsre = RegexBuilder::new(r"(^pub mod .+?$)").multi_line(true).build().unwrap();
    are_captures_sorted(modsre.captures_iter(&librs), "lib.rs")
}

#[test]
fn test_cargotoml() -> Result<(), String> {
    let cargotoml = std::fs::read_to_string("Cargo.toml").unwrap();
    let blocksre = RegexBuilder::new(r"^# default features\r?\n((:?^.+?\r?\n)*)\r?\n# additional features\r?\n((:?^.+?\r?\n)*)").multi_line(true).build().unwrap();
    let linesre = RegexBuilder::new(r"^\*(.+?)$").multi_line(true).build().unwrap();
    let blocks = blocksre.captures(&cargotoml).unwrap();
    are_captures_sorted(linesre.captures_iter(blocks.get(1).unwrap().as_str()), "Cargo.toml default features")?;
    are_captures_sorted(linesre.captures_iter(blocks.get(3).unwrap().as_str()), "Cargo.toml additional features")
}
