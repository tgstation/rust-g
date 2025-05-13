use std::fs;
use std::process::{Command, Output};

#[cfg(feature = "git")]
#[test]
fn git() {
    run_dm_tests("git");
}

#[cfg(feature = "toml")]
#[test]
fn toml() {
    run_dm_tests("toml");
}

#[cfg(feature = "url")]
#[test]
fn url() {
    run_dm_tests("url");
}

#[cfg(feature = "hash")]
#[test]
fn hash() {
    run_dm_tests("hash");
}


fn run_dm_tests(name: &str) {
    let files_data = prepare_all_dmsrc_files();

    std::env::remove_var("RUST_BACKTRACE");

    let byond_bin = std::env::var("BYOND_BIN").expect("environment variable BYOND_BIN");
    let byondexec = format!("{byond_bin}/byondexec");
    let dream_maker = format!("{byond_bin}/DreamMaker");
    let dream_daemon = format!("{byond_bin}/DreamDaemon");

    let dme = format!("tests/dm/{name}.dme");
    let dmb = format!("tests/dm/{name}.dmb");

    let target_dir = if cfg!(target_os = "linux") {
        "i686-unknown-linux-gnu"
    } else {
        "i686-pc-windows-gnu"
    };
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let fname = if cfg!(target_os = "linux") {
        "librust_g.so"
    } else {
        "rust_g.dll"
    };
    let rust_g = format!("target/{target_dir}/{profile}/{fname}");

    let output = Command::new("bash")
        .arg(&byondexec)
        .arg(&dream_maker)
        .arg(&dme)
        .output()
        .unwrap();
    dump(&output);
    generic_check(&output);

    let output = Command::new("bash")
        .arg(&byondexec)
        .arg(&dream_daemon)
        .arg(&dmb)
        .arg("-trusted")
        .arg("-cd")
        .arg(env!("CARGO_MANIFEST_DIR"))
        .env("RUST_G", &rust_g)
        .output()
        .unwrap();
    let _ = std::fs::remove_file(&dmb);
    dump(&output);
    generic_check(&output);
    runtime_check(&output);

    revert_all_dmsrc_files(files_data);
}

fn prepare_all_dmsrc_files() -> Vec<(String, String)> {
    println!("Current working directory: {}", std::env::current_dir().unwrap().display());
    println!("Reading from 'dmsrc' directory...");
    let mut files_data = Vec::new();
    for entry in fs::read_dir("dmsrc").unwrap() {
        let path = entry.unwrap().path();

        if path.extension().and_then(|s| s.to_str()) == Some("dm") {
            let path_str = path.to_string_lossy().to_string();
            let original = fs::read_to_string(&path_str).unwrap();

            // Uncomment special sections
            let stripped = original.replace("/***", "").replace("***/", "");
            fs::write(&path_str, &stripped).unwrap();

            files_data.push((path_str.clone(), original));
            println!("Found DM file: {}", path_str);
        }
    }

    files_data
}

fn revert_all_dmsrc_files(files_data: Vec<(String, String)>) {
    println!("Reverting changes to DM files...");
    for (path, original) in files_data {
        fs::write(&path, &original).unwrap();
        println!("Reverted: {}", path);
    }
}

fn dump(output: &Output) {
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
}

fn generic_check(output: &Output) {
    if !output.status.success() {
        panic!("process exited with {:?}", output.status);
    }
}

fn runtime_check(output: &Output) {
    for line in output.stderr.split(|&c| c == b'\n') {
        if line.starts_with(b"runtime error: ") {
            panic!("{}", String::from_utf8_lossy(line));
        }
    }
}
