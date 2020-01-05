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

byond_fn! { log_write(path, data, format) {
    FILE_MAP.with(|cell| -> Result<()> {
        // open file
        let mut map = cell.borrow_mut();
        let path = Path::new(&path as &str);
        let file = match map.entry(filename(path)?) {
            Entry::Occupied(elem) => elem.into_mut(),
            Entry::Vacant(elem) => elem.insert(open(path)?),
        };

        if let Some(format_bool) = format.parse() {
        if format_bool {
            // write first line, timestamped
            let mut iter = data.split('\n');
            if let Some(line) = iter.next() {
                write!(file, "[{}] {}\n", Utc::now().format("%F %T%.3f"), line)?;
            }

            // write remaining lines
            for line in iter {
                write!(file, " - {}\n", line)?;
            }
        } else {
            write!(file, "{}", data)?;
        }

        Ok(())
    }).err()
} }

byond_fn! { log_close_all()! {
    FILE_MAP.with(|cell| {
        let mut map = cell.borrow_mut();
        map.clear();
    })
} }

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
