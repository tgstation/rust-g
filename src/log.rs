use crate::error::{Error, Result};
use chrono::{Local, Utc};
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::hash_map::{Entry, HashMap},
    ffi::OsString,
    fs,
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
    str::FromStr,
};

thread_local! {
    static FILE_MAP: RefCell<HashMap<OsString, File>> = RefCell::new(HashMap::new());
}

// writes the log to a file, with a timestamp in the UTC timezone
byond_fn!(fn log_write(path, data, ...rest) {
    write_log(path, data, rest)
});

fn write_log(path: &str, data: &str, rest: &[Cow<'_, str>]) -> Option<Error> {
    FILE_MAP
        .with(|cell| -> Result<()> {
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
                let timezone_string = rest.last().map(|x| &**x).unwrap_or("UTC");
                if let Some(line) = iter.next() {
                    let mut timestamp = Local::now().format("%F %T%.3f");
                    if timezone_string != "Local" {
                        let timezone =
                            chrono_tz::Tz::from_str(timezone_string).unwrap_or(chrono_tz::UTC);
                        timestamp = Utc::now().with_timezone(&timezone).format("%F %T%.3f");
                    }
                    writeln!(file, "[{}] {}", timestamp, line)?;
                }

                // write remaining lines
                for line in iter {
                    writeln!(file, " - {line}")?;
                }
            }

            Ok(())
        })
        .err()
}

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
