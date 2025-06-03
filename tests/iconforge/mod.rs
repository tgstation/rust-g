use super::run_dm_tests;
use dmi::{
    error::DmiError,
    icon::{Icon, IconState},
};
use image::{DynamicImage, GenericImageView};
use std::{
    fs::{read_dir, File},
    io::BufReader,
    path::Path,
};

#[test]
fn iconforge() {
    // Generate icons for comparison
    run_dm_tests("iconforge", false);
    // Compare said icons
    std::env::set_var("RUST_BACKTRACE", "1");
    let mut differences: Vec<String> = Vec::new();
    let tmp_files = read_dir("tests/dm/tmp/").unwrap();
    for tmp_entry in tmp_files {
        if let Ok(entry) = tmp_entry {
            if let Some(file_name) = entry.file_name().to_str() {
                if !file_name.starts_with("dm_") || !file_name.ends_with(".dmi") {
                    continue;
                }
                let size = file_name.replace("dm_", "").replace(".dmi", "");
                let rustg_path_str = format!("tests/dm/tmp/rustg_{size}.dmi");
                let rustg_path = Path::new(&rustg_path_str);
                if !std::fs::exists(rustg_path).unwrap() {
                    panic!("Could not find corresponding rustg_{size}.dmi for dm_{size}.dmi!")
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
    }
    if !differences.is_empty() {
        panic!(
            "icons were found to differ:\n\n---\n{}",
            differences.join("\n\n---\n")
        )
    } else {
        println!("no icons differ!");
    }
}

fn compare_dmis(dm_path: &Path, rustg_path: &Path) -> Option<String> {
    println!(
        "Comparing {} and {}",
        dm_path.display(),
        rustg_path.display()
    );
    let mut differences: Vec<String> = Vec::new();
    let dm_icon = dmi_from_path(dm_path).unwrap();
    let rustg_icon = dmi_from_path(rustg_path).unwrap();
    for dm_state in &dm_icon.states {
        if let Some(rustg_state) = rustg_icon
            .states
            .iter()
            .find(|rustg_state| rustg_state.name == dm_state.name)
        {
            if let Some(diff) = compare_states(dm_state, rustg_state) {
                differences.push(format!("icon state {}:\n{diff}\n", dm_state.name));
            }
        } else {
            differences.push(format!(
                "icon state {}:\ndoes not exist in rustg\n",
                dm_state.name
            ));
        }
    }
    for rustg_state in &rustg_icon.states {
        if let None = dm_icon
            .states
            .iter()
            .find(|dm_state| dm_state.name == rustg_state.name)
        {
            differences.push(format!(
                "icon state {}:\ndoes not exist in dm",
                rustg_state.name
            ));
        }
    }
    if differences.is_empty() {
        None
    } else {
        Some(differences.join("\n"))
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
            "REWING FLAG: dm: {} - rustg: {}",
            dm_state.rewind, rustg_state.rewind
        ));
    }

    if dm_state.loop_flag != rustg_state.loop_flag {
        differences.push(format!(
            "LOOP FLAG: dm: {:?} - rustg: {:?}",
            dm_state.loop_flag, rustg_state.loop_flag
        ));
    }

    let dm_images_len = dm_state.images.len();
    let rustg_images_len = rustg_state.images.len();
    if dm_images_len != rustg_images_len {
        differences.push(format!(
            "IMAGE COUNT: dm: {} - rustg: {}",
            dm_images_len, rustg_images_len
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
    dm_images: &Vec<DynamicImage>,
    rustg_images: &Vec<DynamicImage>,
    dirs: u8,
) {
    let mut image_index = 0;
    for (dm_image, rustg_image) in std::iter::zip(dm_images, rustg_images) {
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
            differences.push(format!(
                "{} at pixels: {all_coordinates}",
                image_name_from_index(image_index, dirs)
            ));
        }
        image_index += 1;
    }
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
