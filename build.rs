//! Buildscript which will save a `rust_g.dm` with the DLL's public API.

use std::{fs::File, io::Write};

macro_rules! feature_dm_file {
    ($name:expr) => {
        &"dmsrc/{}.dm".replace("{}", $name)
    };
}

macro_rules! feature_dm_exists {
    ($name:expr) => {
        std::path::Path::new(feature_dm_file!($name)).exists()
    };
}

fn main() {
    let mut f = File::create("target/rust_g.dm").unwrap();

    // header
    writeln!(
        f,
        "{}",
        std::fs::read_to_string(feature_dm_file!("main")).unwrap()
    )
    .unwrap();

    for (key, _value) in std::env::vars() {
        // CARGO_FEATURE_<name> — For each activated feature of the package being built, this environment variable will be present where <name> is the name of the feature uppercased and having - translated to _.
        if let Some(uprfeature) = key.strip_prefix("CARGO_FEATURE_") {
            let feature = uprfeature.to_lowercase().replace("_", "-"); // actual proper name of the enabled feature
            if feature_dm_exists!(&feature) {
                writeln!(f, "{}", std::fs::read_to_string(feature_dm_file!(&feature)).unwrap()).unwrap();
            }
        }
    }

    // module: zip
    if enabled!("UNZIP") {
        write!(f, r#"
#define rustg_unzip_download_async(url, unzip_directory) call(RUST_G, "unzip_download_async")(url, unzip_directory)
#define rustg_unzip_check(job_id) call(RUST_G, "unzip_check")("[job_id]")
"#).unwrap();
    }
}
