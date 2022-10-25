#define rustg_unzip_download_async(url, unzip_directory) RGCALL(RUST_G, "unzip_download_async")(url, unzip_directory)
#define rustg_unzip_check(job_id) RGCALL(RUST_G, "unzip_check")("[job_id]")
