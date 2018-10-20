use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Write};

use error::Result;

byond_fn! { file_read(path) {
    read(path).ok()
} }

byond_fn! { file_write(data, path) {
    write(data, path).err()
} }

byond_fn! { file_append(data, path) {
    append(data, path).err()
} }

fn read(path: &str) -> Result<String> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;

    let mut content = String::with_capacity(metadata.len() as usize);
    file.read_to_string(&mut content)?;

    Ok(content)
}

fn write(data: &str, path: &str) -> Result<usize> {
    let mut file = File::create(path)?;

    Ok(file.write(data.as_bytes())?)
}

fn append(data: &str, path: &str) -> Result<usize> {
    let mut file = OpenOptions::new().append(true).create(true).open(path)?;

    Ok(file.write(data.as_bytes())?)
}
