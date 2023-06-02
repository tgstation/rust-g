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

thread_local! {
    static FILE_MAP: RefCell<HashMap<OsString, File>> = RefCell::new(HashMap::new());
}

byond_fn!(fn log_write(path, data, ...rest) {
    FILE_MAP.with(|cell| -> Result<()> {
        // open file
        let mut map = cell.borrow_mut();
        let path = Path::new(path as &str);
        let file = match map.entry(path.into()) {
            Entry::Occupied(elem) => elem.into_mut(),
            Entry::Vacant(elem) => elem.insert(open(path)?),
        };

        if rest.first().map(|x| &**x) == Some("false") {
            // Write the data to the file with no accoutrements.
            write!(file, "{data}")?;
        } else {
            // write first line, timestamped
            let mut iter = data.split('\n');
            if let Some(line) = iter.next() {
                writeln!(file, "[{}] {}", Utc::now().format("%F %T%.3f"), line)?;
            }

            // write remaining lines
            for line in iter {
                writeln!(file, " - {line}")?;
            }
        }

        Ok(())
    }).err()
});

byond_fn!(
    fn log_close_all() {
        FILE_MAP.with(|cell| {
            let mut map = cell.borrow_mut();
            map.clear();
        });
        Some("")
    }
);

fn open(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(OpenOptions::new().append(true).create(true).open(path)?)
}
