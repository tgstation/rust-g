use crate::{error::Result, http::HTTP_CLIENT, jobs};
use std::fs;
use std::io::Write;
use std::path::Path;
use zip::ZipArchive;

struct UnzipPrep {
    req: ureq::RequestBuilder<ureq::typestate::WithoutBody>,
    unzip_directory: String,
}

fn construct_unzip(url: &str, unzip_directory: &str) -> UnzipPrep {
    let req = HTTP_CLIENT.get(url);
    let dir_copy = unzip_directory.to_string();

    UnzipPrep {
        req,
        unzip_directory: dir_copy,
    }
}

byond_fn!(fn unzip_download_async(url, unzip_directory) {
    let unzip = construct_unzip(url, unzip_directory);
    Some(jobs::start(move ||
        do_unzip_download(unzip).unwrap_or_else(|e| e.to_string())
    ))
});

fn do_unzip_download(prep: UnzipPrep) -> Result<String> {
    let unzip_path = Path::new(&prep.unzip_directory);
    let response = prep.req.call().map_err(Box::new)?;

    const LIMIT: u64 = 100 * 1024 * 1024; // 100MB
    let content_length: u64 = response
        .headers()
        .get("Content-Length")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    if content_length > LIMIT {
        return Err(crate::error::Error::HttpTooBig);
    }
    let mut binding = response.into_body();
    let body = binding.with_config().limit(LIMIT);
    let content = body
        .read_to_vec()
        .map_err(|e| crate::error::Error::HttpParse(e.to_string()))?;

    let reader = std::io::Cursor::new(content);
    let mut archive = ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;

        let file_path = unzip_path.join(entry.name());

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?
        }

        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)?;

        let mut writer = std::io::BufWriter::new(file);
        std::io::copy(&mut entry, &mut writer)?;
        writer.flush()?;
    }

    Ok("true".to_string())
}

byond_fn!(fn unzip_check(id) {
    Some(jobs::check(id))
});
