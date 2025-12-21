use std::fs;
use std::path::Path;
use std::process::{Command, Output};

#[cfg(feature = "git")]
#[test]
fn git() {
    run_dm_tests("git", true);
}

#[cfg(feature = "toml")]
#[test]
fn toml() {
    run_dm_tests("toml", false);
}

#[cfg(feature = "url")]
#[test]
fn url() {
    run_dm_tests("url", false);
}

#[cfg(feature = "hash")]
#[test]
fn hash() {
    run_dm_tests("hash", false);
}

#[cfg(feature = "dice")]
#[test]
fn dice() {
    run_dm_tests("dice", false);
}

#[cfg(feature = "iconforge")]
pub mod iconforge;

/**
 * Find a valid BYOND bin path on the system.
 */
fn find_byond() -> String {
    match std::env::var("BYOND_BIN") {
        Ok(bin) => bin,
        Err(_) => {
            let paths = vec![
                "C:/Program Files (x86)/BYOND/bin",
                "C:/Program Files/BYOND/bin",
            ];
            let mut found_path = None;
            for path in paths {
                if let Ok(exists) = fs::exists(Path::new(path)) {
                    if exists {
                        found_path = Some(path.to_string());
                        break;
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            found_path.expect("Could not find environment variable BYOND_BIN, or any valid installation path for BYOND.")
        }
    }
}

fn use_byond_executable<F>(byond_bin: &str, windows: &str, linux: &str, command: F) -> Output
where
    F: Fn(&mut Command) -> &mut Command,
{
    if cfg!(target_os = "linux") {
        let byondexec = format!("{byond_bin}/byondexec");
        let linux_full = format!("{byond_bin}/{linux}");
        command(Command::new("bash").arg(&byondexec).arg(&linux_full))
            .output()
            .unwrap()
    } else {
        let windows_full = format!("{byond_bin}/{windows}.exe");
        let path = Path::new(&windows_full);
        if !fs::exists(path).unwrap_or(false) {
            panic!(
                "BYOND executable not found at: {}\nBYOND_BIN is set to: {}\nPlease install BYOND or set the BYOND_BIN environment variable correctly.",
                windows_full, byond_bin
            );
        }
        command(&mut Command::new(&windows_full))
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute BYOND command: {}", windows_full))
    }
}

fn compile_and_run_dme(name: &str, rust_g_lib_path: &str, chdir: Option<&str>) -> Output {
    let byond_bin = find_byond();

    let dme = format!("tests/dm/{name}.dme");
    let dmb = format!("tests/dm/{name}.dmb");

    let output = use_byond_executable(&byond_bin, "dm", "DreamMaker", |c| c.arg(&dme));
    dump(&output);
    generic_check(&output);

    let output = use_byond_executable(&byond_bin, "dd", "DreamDaemon", |c| {
        c.arg(&dmb)
            .arg("-trusted")
            .arg("-cd")
            .arg(chdir.unwrap_or("."))
            .env("RUST_G", rust_g_lib_path)
    });

    // Cleanup
    let _ = std::fs::remove_file(&dmb);
    let _ = std::fs::remove_file(format!("tests/dm/{name}.rsc"));
    let _ = std::fs::remove_file(format!("tests/dm/{name}.dyn.rsc"));
    let _ = std::fs::remove_file(format!("tests/dm/{name}.lk"));
    let _ = std::fs::remove_file(format!("tests/dm/{name}.int"));

    dump(&output);
    generic_check(&output);
    output
}

/**
 * Find the rust_g binary and DMSRC and copy them into the test run directory
 */
fn find_and_copy_rustg_lib() -> (String, &'static str, &'static str) {
    let target_dir = if cfg!(target_os = "linux") {
        "i686-unknown-linux-gnu"
    } else {
        //TODO: Remove when win7 is finally deleted like it deserves
        // Try both i686-win7-windows-msvc and i686-pc-windows-msvc
        let win7_target = "i686-win7-windows-msvc";
        let pc_target = "i686-pc-windows-msvc";

        let profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };

        let win7_path = format!("target/{win7_target}/{profile}/rust_g.dll");
        let pc_path = format!("target/{pc_target}/{profile}/rust_g.dll");

        if fs::exists(Path::new(&win7_path)).unwrap_or(false) {
            win7_target
        } else if fs::exists(Path::new(&pc_path)).unwrap_or(false) {
            pc_target
        } else {
            // Default to pc target for error message
            pc_target
        }
    };
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let rustg_lib_fname = if cfg!(target_os = "linux") {
        "librust_g.so"
    } else {
        "rust_g.dll"
    };
    let rustg_lib_source_path = format!("target/{target_dir}/{profile}/{rustg_lib_fname}");
    println!("Source RUST_G path: {rustg_lib_source_path}");
    match fs::exists(Path::new(&rustg_lib_source_path)) {
        Ok(exists) => {
            if !exists {
                panic!("Source RUST_G path does not exist! Try rebuilding the project with the corresponding target and debug or release mode.")
            }
        }
        Err(err) => panic!("Error accessing source rust_g path! {err}"),
    }
    let rustg_lib_path = format!("tests/dm/{rustg_lib_fname}");

    // Try to copy, but if it fails, check if the file already exists and matches
    match fs::copy(&rustg_lib_source_path, &rustg_lib_path) {
        Ok(_) => println!("Successfully copied {rustg_lib_fname}"),
        Err(e) => {
            println!("Copy failed with error: {:?}", e);
            println!("Error kind: {:?}", e.kind());
            println!("Checking if destination already exists...");

            if fs::exists(Path::new(&rustg_lib_path)).unwrap_or(false) {
                println!("Destination file already exists, will use it");
                // File exists, assume it's correct (may be from another test running in parallel)
            } else {
                panic!(
                    "Failed to copy {} to {}: {:?}",
                    rustg_lib_source_path, rustg_lib_path, e
                );
            }
        }
    }

    let rustg_dm_path = "tests/dm/rust_g.dm";
    let rustg_dm_source = "target/rust_g.dm";
    if !fs::exists(Path::new(rustg_dm_source)).unwrap_or(false) {
        panic!(
            "rust_g.dm source file does not exist at: {}",
            rustg_dm_source
        );
    }

    match fs::copy(rustg_dm_source, rustg_dm_path) {
        Ok(_) => println!("Successfully copied rust_g.dm"),
        Err(e) => {
            println!("Copy rust_g.dm failed with error: {:?}", e);
            println!("Error kind: {:?}", e.kind());

            if fs::exists(Path::new(rustg_dm_path)).unwrap_or(false) {
                println!("Destination rust_g.dm already exists, will use it");
            } else {
                panic!(
                    "Failed to copy {} to {}: {:?}",
                    rustg_dm_source, rustg_dm_path, e
                );
            }
        }
    }

    (rustg_lib_path, rustg_lib_fname, rustg_dm_path)
}

fn run_dm_tests(name: &str, use_repo_root: bool) {
    std::env::remove_var("RUST_BACKTRACE");

    let (rustg_lib_path, rustg_lib_fname, rustg_dm_path) = find_and_copy_rustg_lib();

    let output = compile_and_run_dme(
        name,
        if use_repo_root {
            &rustg_lib_path
        } else {
            rustg_lib_fname
        },
        if use_repo_root {
            Some(env!("CARGO_MANIFEST_DIR"))
        } else {
            None
        },
    );
    runtime_check(&output);

    // Cleanup
    let _ = std::fs::remove_file(&rustg_lib_path);
    let _ = std::fs::remove_file(rustg_dm_path);
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
