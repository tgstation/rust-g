use crate::{error::Result, http::HTTP_CLIENT, jobs};
use zip::ZipArchive;

struct UnzipPrep {
    req: ureq::Request,
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
    let response = prep.req.send_bytes(&[]).map_err(Box::new)?;

    let mut content = Vec::new();
    response.into_reader().read_to_end(&mut content)?;

    let reader = std::io::Cursor::new(content);
    let mut archive = ZipArchive::new(reader)?;
    // Fails on entries that would land outside the directory.
    archive.extract(&prep.unzip_directory)?;

    Ok("true".to_string())
}

byond_fn!(fn unzip_check(id) {
    Some(jobs::check(id))
});
