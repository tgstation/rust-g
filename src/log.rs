use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::ffi::OsString;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

use chrono::Utc;

use error::{Error, Result};

thread_local! {
    static FILE_MAP: RefCell<HashMap<OsString, File>> = RefCell::new(HashMap::new());
}

byond_function! { log_write(path, data) {
    data.split('\n')
        .map(|line| format(line))
        .map(|line| write(path, line))
        .collect::<Result<Vec<_>>>()
        .err()
} }

byond_function! { log_close_all()! {
    FILE_MAP.with(|cell| {
        let mut map = cell.borrow_mut();
        map.clear();
    })
} }

fn format(data: &str) -> String {
    format!("[{}] {}\n", Utc::now().format("%F %T%.3f"), data)
}

fn write(path: &str, data: String) -> Result<usize> {
    FILE_MAP.with(|cell| {
        let mut map = cell.borrow_mut();
        let path = Path::new(path);
        let file = match map.entry(filename(path)?) {
            Entry::Occupied(elem) => elem.into_mut(),
            Entry::Vacant(elem) => elem.insert(open(path)?),
        };

        Ok(file.write(&data.into_bytes())?)
    })
}

fn filename(path: &Path) -> Result<OsString> {
    match path.file_name() {
        Some(filename) => Ok(filename.to_os_string()),
        None => Err(Error::InvalidFilename),
    }
}

fn open(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?
    }

    Ok(OpenOptions::new().append(true).create(true).open(path)?)
}
