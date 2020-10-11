use crate::error::Result;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
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
    let mut file = File::open(path)?;
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

    let mut file = File::create(path)?;

    Ok(file.write(data.as_bytes())?)
}

fn append(data: &str, path: &str) -> Result<usize> {
    let path: &std::path::Path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new().append(true).create(true).open(path)?;

    Ok(file.write(data.as_bytes())?)
}
