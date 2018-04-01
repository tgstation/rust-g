use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

use chrono::Utc;
use failure::Error;

thread_local! {
    static FILE_MAP: RefCell<HashMap<String, File>> = RefCell::new(HashMap::new());
}

fn timestamped(data: &str) -> String {
    format!("[{}] {}", Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"), data)
}

fn write(filename: &str, data: String) -> Result<usize, Error> {
    let mut data = data;
    data.push('\n');

    FILE_MAP.with(|cell| {
        let path = Path::new(filename);
        let filename = match path.file_name() {
            Some(filename) => filename.to_string_lossy().into_owned(),
            None => return Err(format_err!("invalid or empty filename")),
        };

        let mut map = cell.borrow_mut();
        let file = match map.entry(filename) {
            Occupied(elem) => elem.into_mut(),
            Vacant(elem) => {
                match path.parent() {
                    Some(parent) => fs::create_dir_all(parent)?,
                    None => {}
                };

                let file = OpenOptions::new().append(true).create(true).open(path)?;
                elem.insert(file)
            }
        };

        Ok(file.write(&data.into_bytes())?)
    })
}

fn close() {
    FILE_MAP.with(|cell| {
        let mut map = cell.borrow_mut();
        map.clear();
    });
}

byond_function! { log_write(filename, data) {
    let result: Result<Vec<_>, Error> = data.split("\n")
        .map(|line| write(filename, timestamped(line)))
        .collect();

    match result {
        Ok(_) => None,
        Err(err) => Some(err.to_string()),
    }
} }

byond_function! { log_close_all() {
    close();

    None
} }
