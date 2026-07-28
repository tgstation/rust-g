use crate::error::Result;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Read, Write},
};

byond_fn!(fn file_read(path) {
    read(path).ok()
});

byond_fn!(fn file_exists(path) {
    Some(exists(path))
});

byond_fn!(fn file_write(data, path) {
    write(data, path).err()
});

byond_fn!(fn file_append(data, path) {
    append(data, path).err()
});

byond_fn!(fn file_get_line_count(path) {
    Some(get_line_count(path).ok()?.to_string())
});

byond_fn!(fn file_seek_line(path, line) {
    seek_line(path, match line.parse::<usize>() {
        Ok(line) => line,
        Err(_) => return None,
    })
});

fn read(path: &str) -> Result<String> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let mut file = BufReader::new(file);

    let mut content = String::with_capacity(metadata.len() as usize);
    file.read_to_string(&mut content)?;
    let content = content.replace('\r', "");

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

    // basically, this is just like, in case the filesystem has an issue, we won't nuke the original file if there is one
    // write to tmp file first - then rename it to original filename if everything succeeds
    let tmp_path = path.with_extension("tmp");
    let mut file = File::create(&tmp_path)?;
    file.write_all(data.as_bytes())?;
    file.sync_all()?;

    std::fs::rename(&tmp_path, path)?;
    Ok(data.len())
}

fn append(data: &str, path: &str) -> Result<usize> {
    let path: &std::path::Path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = BufWriter::new(OpenOptions::new().append(true).create(true).open(path)?);
    let written = file.write(data.as_bytes())?;

    file.flush()?;
    file.into_inner()
        .map_err(|e| std::io::Error::new(e.error().kind(), e.error().to_string()))?
        .sync_all()?;

    Ok(written)
}

fn get_line_count(path: &str) -> Result<u32> {
    let file = BufReader::new(File::open(path)?);
    Ok(file.lines().count() as u32)
}

fn seek_line(path: &str, line: usize) -> Option<String> {
    let file = BufReader::new(File::open(path).ok()?);
    file.lines().nth(line).and_then(std::result::Result::ok)
}
