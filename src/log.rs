use crate::error::Result;
use chrono::Utc;
use std::{
    cell::RefCell,
    collections::hash_map::{Entry, HashMap},
    ffi::OsString,
    fs,
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use byond_fn::byond_fn;

thread_local! {
    static FILE_MAP: RefCell<HashMap<OsString, File>> = RefCell::new(HashMap::new());
}

#[byond_fn]
fn log_write(path: &Path, data: &str, with_format: Option<bool>) -> Result<()> {
    FILE_MAP.with(|cell| {
        // open file
        let mut map = cell.borrow_mut();
        let file = match map.entry(path.into()) {
            Entry::Occupied(elem) => elem.into_mut(),
            Entry::Vacant(elem) => elem.insert(open(path)?),
        };

        if with_format.unwrap_or(true) {
            // write first line, timestamped
            let mut iter = data.split('\n');
            if let Some(line) = iter.next() {
                let time = Utc::now().format("%F %T%.3f");
                writeln!(file, "[{time}] {line}")?;
            }

            // write the rest of the lines
            for line in iter {
                writeln!(file, "{line}")?;
            }
        } else {
            // Write the data to the file with no accoutrement's.
            write!(file, "{data}")?;
        }
        Ok(())
    })
}

#[byond_fn]
fn log_close_all() {
    FILE_MAP.with(|cell| {
        let mut map = cell.borrow_mut();
        map.clear();
    });
}

fn open(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?
    }

    Ok(OpenOptions::new().append(true).create(true).open(path)?)
}
