use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use std::path::Path;

use chrono::Utc;

thread_local! {
    static FILE_MAP: RefCell<HashMap<String, File>> = RefCell::new(HashMap::new());
}

byond_function! { log_write(path, data) {
    data.split('\n')
        .map(|line| format(line))
        .map(|line| write(path, line))
        .collect::<Result<Vec<_>, Error>>()
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

fn write(path: &str, data: String) -> Result<usize, Error> {
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

fn filename(path: &Path) -> Result<String, Error> {
    match path.file_name() {
        Some(filename) => Ok(filename.to_string_lossy().into_owned()),
        None => Err(Error::Filename),
    }
}

fn open(path: &Path) -> Result<fs::File, io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?
    }

    OpenOptions::new().append(true).create(true).open(path)
}

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "invalid or empty filename")]
    Filename,
    #[fail(display = "io error: {}", _0)]
    Io(#[cause] io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<Error> for String {
    fn from(error: Error) -> String {
        error.to_string()
    }
}

impl From<Error> for Vec<u8> {
    fn from(error: Error) -> Vec<u8> {
        error.to_string().into_bytes()
    }
}
