use super::run_dm_tests;
use dmi::{
    error::DmiError,
    icon::{Icon, IconState},
};
use image::RgbaImage;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    fs::{File, read_dir},
    io::BufReader,
    path::Path,
    sync::{Arc, Mutex},
};

#[test]
fn iconforge() {
    tmp_cleanup();
    // Generate icons for comparison
    run_dm_tests("iconforge", false);
    // Compare said icons
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    let mut differences: Vec<String> = Vec::new();
    for entry in read_dir("tests/dm/tmp/").unwrap().flatten() {
        if let Some(file_name) = entry.file_name().to_str() {
            if !file_name.starts_with("iconforge_dm_") || !file_name.ends_with(".dmi") {
                continue;
            }
            let size = file_name.replace("iconforge_dm_", "").replace(".dmi", "");
            let rustg_path_str = format!("tests/dm/tmp/iconforge_rustg_{size}.dmi");
            let rustg_path = Path::new(&rustg_path_str);
            if !std::fs::exists(rustg_path).unwrap() {
                panic!(
                    "Could not find corresponding iconforge_rustg_{size}.dmi for iconforge_dm_{size}.dmi!"
                )
            }
            if let Some(diff) = compare_dmis(entry.path().as_path(), rustg_path) {
                differences.push(format!(
                    "icon {} differs from {}:\n{}",
                    rustg_path.display(),
                    entry.path().display(),
                    diff
                ));
            }
        }
    }
    // Compare headless icons with non-headless icons for sanity check
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    let mut differences: Vec<String> = Vec::new();
    for entry in read_dir("tests/dm/tmp/").unwrap().flatten() {
        if let Some(file_name) = entry.file_name().to_str() {
            if !file_name.starts_with("iconforge_rustg_") || !file_name.ends_with(".dmi") {
                continue;
            }
            let size = file_name
                .replace("iconforge_rustg_", "")
                .replace(".dmi", "");
            let headless_path_str = format!("tests/dm/tmp/headless_iconforge_rustg_{size}.dmi");
            let headless_path = Path::new(&headless_path_str);
            if !std::fs::exists(headless_path).unwrap() {
                panic!(
                    "Could not find corresponding headless_iconforge_rustg_{size}.dmi for iconforge_rustg_{size}.dmi!"
                )
            }
            if let Some(diff) = compare_dmis(entry.path().as_path(), headless_path) {
                differences.push(format!(
                    "icon (headless) {} differs from {}:\n{}",
                    headless_path.display(),
                    entry.path().display(),
                    diff
                ));
            }
        }
    }
    // Compare BYOND's copied version of a valid headless icon states
    if let Some(diff) = compare_dmis(
        Path::new("tests/dm/tmp/iconforge_valid_headless_copied.dmi"),
        Path::new("tests/dm/tmp/iconforge_valid_headless.dmi"),
    ) {
        differences.push(format!(
            "icon tests/dm/tmp/iconforge_valid_headless.dmi differs from tests/dm/tmp/iconforge_valid_headless_copied.dmi:\n{diff}",
        ));
    }
    // Compare gags icons
    if let Some(diff) = compare_dmis(
        Path::new("tests/dm/rsc/iconforge_gags_dm.dmi"),
        Path::new("tests/dm/tmp/iconforge_gags_rustg.dmi"),
    ) {
        differences.push(format!(
            "icon tests/dm/tmp/iconforge_gags_rustg.dmi differs from tests/dm/rsc/iconforge_gags_dm.dmi:\n{diff}"
        ));
    }
    if !differences.is_empty() {
        panic!(
            "icons were found to differ:\n\n---\n{}",
            differences.join("\n\n---\n")
        )
    } else {
        println!("no icons differ!");
        tmp_cleanup();
    }
}

fn tmp_cleanup() {
    let dir = match read_dir("tests/dm/tmp/") {
        Ok(dir) => dir,
        Err(_) => {
            let _ = std::fs::create_dir_all("tests/dm/tmp/");
            return;
        }
    };
    for entry in dir.flatten() {
        if let Some(file_name) = entry.file_name().to_str()
            && (file_name.starts_with("iconforge_") || file_name.starts_with("headless_iconforge_"))
            && file_name.ends_with(".dmi")
        {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

fn compare_dmis(dm_path: &Path, rustg_path: &Path) -> Option<String> {
    println!(
        "Comparing {} and {}",
        dm_path.display(),
        rustg_path.display()
    );
    let differences: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let dm_icon = dmi_from_path(dm_path).unwrap();
    let rustg_icon = dmi_from_path(rustg_path).unwrap();
    dm_icon.states.par_iter().for_each(|dm_state| {
        if let Some(rustg_state) = rustg_icon
            .states
            .iter()
            .find(|rustg_state| rustg_state.name == dm_state.name)
        {
            if let Some(diff) = compare_states(dm_state, rustg_state) {
                differences
                    .lock()
                    .unwrap()
                    .push(format!("icon state {}:\n{diff}\n", dm_state.name));
            }
        } else {
            differences.lock().unwrap().push(format!(
                "icon state {}:\ndoes not exist in rustg\n",
                dm_state.name
            ));
        }
    });
    if dm_icon
        .states
        .iter()
        .map(|state| &state.name)
        .collect::<Vec<&String>>()
        != rustg_icon
            .states
            .iter()
            .map(|state| &state.name)
            .collect::<Vec<&String>>()
    {
        differences
            .lock()
            .unwrap()
            .push(String::from("icon state order differs\n"));
    }
    for rustg_state in &rustg_icon.states {
        if !dm_icon
            .states
            .iter()
            .any(|dm_state| dm_state.name == rustg_state.name)
        {
            differences.lock().unwrap().push(format!(
                "icon state {}:\ndoes not exist in dm",
                rustg_state.name
            ));
        }
    }
    let diffs_unlocked = differences.lock().unwrap();
    if diffs_unlocked.is_empty() {
        None
    } else {
        Some(diffs_unlocked.join("\n"))
    }
}

fn compare_states(dm_state: &IconState, rustg_state: &IconState) -> Option<String> {
    let mut differences: Vec<String> = Vec::new();

    if dm_state.dirs != rustg_state.dirs {
        differences.push(format!(
            "DIRS: dm: {} - rustg: {}",
            dm_state.dirs, rustg_state.dirs
        ));
    }

    if dm_state.frames != rustg_state.frames {
        differences.push(format!(
            "FRAMES: dm: {} - rustg: {}",
            dm_state.frames, rustg_state.frames
        ));
    }

    if dm_state.movement != rustg_state.movement {
        differences.push(format!(
            "MOVEMENT FLAG: dm: {} - rustg: {}",
            dm_state.movement, rustg_state.movement
        ));
    }

    if dm_state.rewind != rustg_state.rewind {
        differences.push(format!(
            "REWIND FLAG: dm: {} - rustg: {}",
            dm_state.rewind, rustg_state.rewind
        ));
    }

    if dm_state.loop_flag != rustg_state.loop_flag {
        differences.push(format!(
            "LOOP FLAG: dm: {:?} - rustg: {:?}",
            dm_state.loop_flag, rustg_state.loop_flag
        ));
    }

    if dm_state.delay != rustg_state.delay {
        differences.push(format!(
            "DELAYS: dm: {:?} - rustg: {:?}",
            dm_state.delay, rustg_state.delay
        ));
    }

    let dm_images_len = dm_state.images.len();
    let rustg_images_len = rustg_state.images.len();
    if dm_images_len != rustg_images_len {
        differences.push(format!(
            "IMAGE COUNT: dm: {dm_images_len} - rustg: {rustg_images_len}"
        ));
    } else {
        compare_images(
            &mut differences,
            &dm_state.images,
            &rustg_state.images,
            dm_state.dirs,
        );
    }

    if differences.is_empty() {
        None
    } else {
        Some(differences.join("\n"))
    }
}

fn compare_images(
    differences: &mut Vec<String>,
    dm_images: &Vec<RgbaImage>,
    rustg_images: &Vec<RgbaImage>,
    dirs: u8,
) {
    let safe_diffs = Arc::new(Mutex::new(Vec::<String>::new()));
    dm_images.par_iter().zip(rustg_images).enumerate().for_each(
        |(image_index, (dm_image, rustg_image))| {
            let mut image_differences: Vec<String> = Vec::new();
            let mut break_now = false;
            for x in 0..dm_image.width() {
                if break_now {
                    break;
                }
                for y in 0..dm_image.height() {
                    let dm_pixel = dm_image.get_pixel(x, y);
                    let rustg_pixel = rustg_image.get_pixel(x, y);
                    let r_diff = (dm_pixel[0] as i32 - rustg_pixel[0] as i32).abs();
                    let g_diff = (dm_pixel[1] as i32 - rustg_pixel[1] as i32).abs();
                    let b_diff = (dm_pixel[2] as i32 - rustg_pixel[2] as i32).abs();
                    let a_diff = (dm_pixel[3] as i32 - rustg_pixel[3] as i32).abs();
                    // allow off-by-two because literally who can tell
                    if r_diff <= 2 && g_diff <= 2 && b_diff <= 2 && a_diff <= 2 {
                        continue;
                    }
                    // RGB might differ on empty pixels, but it doesn't matter
                    if dm_pixel[3] == 0 && rustg_pixel[3] == 0 {
                        continue;
                    }
                    let mut channels = String::with_capacity(4);
                    channels.push_str(if r_diff > 2 { "R" } else { "#" });
                    channels.push_str(if g_diff > 2 { "G" } else { "#" });
                    channels.push_str(if b_diff > 2 { "B" } else { "#" });
                    channels.push_str(if a_diff > 2 { "A" } else { "#" });
                    if image_differences.len() < 7 {
                        image_differences.push(format!("({x},{y}:{channels})"));
                    } else if image_differences.len() == 7 {
                        image_differences.push(String::from("..."));
                        break_now = true;
                        break;
                    }
                }
            }
            if !image_differences.is_empty() {
                let all_coordinates = image_differences.join(";");
                safe_diffs.lock().unwrap().push(format!(
                    "{} at pixels: {all_coordinates}",
                    image_name_from_index(image_index, dirs)
                ));
            }
        },
    );
    differences.append(&mut safe_diffs.lock().unwrap().clone());
}

fn image_name_from_index(index: usize, dirs: u8) -> String {
    let frame = index / dirs as usize + 1;
    let dir_order = index % dirs as usize;
    let dir = dmi::icon::DIR_ORDERING[dir_order];
    format!("dir={dir} frame={frame}")
}

fn dmi_from_path(path: &Path) -> Result<Icon, DmiError> {
    let icon_file = File::open(path).unwrap();
    let reader = BufReader::new(icon_file);
    Icon::load(reader)
}
