use crate::error::Result;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
};

byond_fn! { file_read(path) {
    read(path).ok()
} }

byond_fn! { file_exists(path) {
    Some(exists(path))
} }

byond_fn! { file_write(data, path) {
    write(data, path).err()
} }

byond_fn! { file_append(data, path) {
    append(data, path).err()
} }

fn read(path: &str) -> Result<String> {
    let mut file = BufReader::new(File::open(path)?);
    let metadata = file.metadata()?;

    let mut content = String::with_capacity(metadata.len() as usize);
    file.read_to_string(&mut content)?;
    let content = content.replace("\r", "");

    Ok(content)
}

fn exists(path: &str) -> String {
    let path = std::path::Path::new(path);
    path.exists().to_string()
}

fn write(data: &str, path: &str) -> Result<usize> {
    let path: &std::path::Path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = BufWriter::new(File::create(path)?);
    let written = file.write(data.as_bytes())?;

    file.flush()?;
    file.into_inner()?.sync_all()?;

    Ok(written)
}

fn append(data: &str, path: &str) -> Result<usize> {
    let path: &std::path::Path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = BufWriter::new(OpenOptions::new().append(true).create(true).open(path)?);
    let written = file.write(data.as_bytes())?;

    file.flush()?;
    file.into_inner()?.sync_all()?;

    Ok(written)
}
