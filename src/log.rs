use crate::error::{Error, Result};
use chrono::{Local, Utc};
use chrono_tz::Tz;
use std::{
    cell::RefCell,
    collections::hash_map::{Entry, HashMap},
    ffi::OsString,
    fs,
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

const TIMESTAMP_FORMAT: &str = "%F %T%.3f";

thread_local! {
    static FILE_MAP: RefCell<HashMap<OsString, File>> = RefCell::new(HashMap::new());
}

byond_fn!(fn log_write(path, data, format, timezone) {
    FILE_MAP.with(|cell| -> Result<()> {
        let timezone: Option<Tz> = match timezone {
            "" => Some(Tz::UTC),
            "Local" => None,
            name => Some(name.parse().map_err(|_| Error::InvalidTimezone(name.to_owned()))?),
        };

        // open file
        let mut map = cell.borrow_mut();
        let path = Path::new(path);
        let file = match map.entry(path.into()) {
            Entry::Occupied(elem) => elem.into_mut(),
            Entry::Vacant(elem) => elem.insert(open(path)?),
        };

        if format == "false" {
            // Write the data to the file with no accoutrements.
            write!(file, "{data}")?;
        } else {
            // write first line, timestamped
            let mut iter = data.split('\n');
            if let Some(line) = iter.next() {
                let timestamp = match timezone {
                    Some(tz) => Utc::now().with_timezone(&tz).format(TIMESTAMP_FORMAT),
                    None => Local::now().format(TIMESTAMP_FORMAT),
                };
                writeln!(file, "[{timestamp}] {line}")?;
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
