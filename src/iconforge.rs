// DMI spritesheet generator
// Developed by itsmeow
use crate::jobs;
use crate::error::Error;
use std::{
    fs::File,
    io::BufReader,
    num::ParseIntError,
};
use dmi::icon::{Icon, IconState};
use image::{DynamicImage, GenericImage, GenericImageView, Pixel, Rgba};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
//use raster::Image;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracy_full::{zone, frame};
use once_cell::sync::Lazy;
static ICON_FILES: Lazy<Mutex<HashMap<String, Icon>>> = Lazy::new(Mutex::default);
static ICON_STATES: Lazy<Mutex<HashMap<String, &mut DynamicImage>>> = Lazy::new(Mutex::default);

const SOUTH: u8 = 2;
const NORTH: u8 = 1;
const EAST: u8 = 4;
const WEST: u8 = 8;
const FOUR_DIRS: [u8; 4] = [SOUTH, NORTH, EAST, WEST];
const SOUTHEAST: u8 = SOUTH | EAST; // 6
const SOUTHWEST: u8 = SOUTH | WEST; // 10
const NORTHEAST: u8 = NORTH | EAST; // 5
const NORTHWEST: u8 = NORTH | WEST; // 9
const EIGHT_DIRS: [u8; 8] = [SOUTH, NORTH, EAST, WEST, SOUTHEAST, SOUTHWEST, NORTHEAST, NORTHWEST];

const DMI_ORDERING: [u8; 8] = EIGHT_DIRS;
// This is an array mapping the DIR number from above to a position in DMIs, such that DIR_TO_INDEX[DIR] = DMI_ORDERING.indexof(DIR)
// 255 is invalid.
const DIR_TO_INDEX: [u8; 11] = [255, 1, 0, 255, 2, 6, 4, 255, 3, 7, 5];


byond_fn!(fn iconforge_generate(file_path, spritesheet_name, sprites) {
    catch_panic(file_path, spritesheet_name, sprites).err()
});


byond_fn!(fn iconforge_generate_async(file_path, spritesheet_name, sprites) {
    // Take ownership before passing
    let file_path = file_path;
    let spritesheet_name = spritesheet_name;
    let sprites = sprites;
    Some(jobs::start(move || {
        match catch_panic(&file_path, &spritesheet_name, &sprites) {
            Ok(o) => o.to_string(),
            Err(e) => e.to_string()
        }
    }))
});

byond_fn!(fn iconforge_check(id) {
    Some(jobs::check(id))
});

#[derive(Serialize)]
struct Returned {
    sizes: Vec<String>,
    sprites: HashMap<String, SpritesheetEntry>,
    error: String,
}

#[derive(Serialize, Clone)]
struct SpritesheetEntry {
    size_id: String,
    position: u32,
}

#[derive(Serialize, Deserialize, Clone)]
struct IconObject {
	icon_file: String,
	icon_state: String,
	dir: u8,
	frame: u32,
	moving: u8,
	transform: Vec<Transform>
}

trait IcoString {
    fn to_icostring() -> String;
}

impl IcoString for IconObject {
    fn to_icostring() -> String {
        return "".to_string(); // TODO implement this as as unique ID. Transforms need another trait
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum Transform {
    BlendColorTransform {
        color: String,
        blend_mode: u8,
    },
    BlendIconTransform {
        icon: IconObject,
        blend_mode: u8,
    },
    ScaleTransform {
        width: u32,
        height: u32,
    },
    CropTransform {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    }
}

fn catch_panic(file_path: &str, spritesheet_name: &str, sprites: &str) -> std::result::Result<String, Error> {
    let x = std::panic::catch_unwind(|| {
        let result = generate_spritesheet(file_path, spritesheet_name, sprites);
        frame!();
        return result;
    });
    if x.is_err() {
        match x.unwrap_err().downcast_ref::<String>() {
            Some(as_string) => {
                return Err(Error::IconState(as_string.to_owned()))
            }
            None => {
                return Err(Error::IconState("Failed to stringify panic".to_string()))
            }
        }
    }
    return x.ok().unwrap()
}

fn generate_spritesheet(file_path: &str, spritesheet_name: &str, sprites: &str) -> std::result::Result<String, Error> {
    zone!("generate_spritesheet");

    let error: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let size_to_images: Arc<Mutex<HashMap<String, Vec<&mut DynamicImage>>>> = Arc::new(Mutex::new(HashMap::new()));
    let sprites_map: HashMap<String, IconObject> = serde_json::from_str::<HashMap<String, IconObject>>(sprites)?;
    let sprites_objects: Arc<Mutex<HashMap<String, SpritesheetEntry>>> = Arc::new(Mutex::new(HashMap::new()));

    // Pre-load all the DMIs now.
    sprites_map.par_iter().for_each(|sprite_entry| {
        zone!("sprite_to_icons");
        let (_, icon) = sprite_entry;
        icon_to_icons(icon).par_iter().for_each(|icon| {
            if let Err(err) = icon_to_dmi(icon) {
                error.lock().unwrap().push(err);
                return;
            }
        });
    });

    // Pick the specific icon states out of the DMI
    sprites_map.par_iter().for_each(|sprite_entry| {
        zone!("map_sprite");
        let (sprite_name, icon) = sprite_entry;

        // get DynamicImage
        let image_result = icon_to_image(icon, sprite_name);
        if image_result.is_err() {
            error.lock().unwrap().push(image_result.unwrap_err());
            return;
        }
        let image = image_result.unwrap();

        // apply transforms here

        for transform in &icon.transform {
            match transform {
                Transform::BlendColorTransform { color, blend_mode } => {
                    let mutator = mutate(*blend_mode);
                    let color_parts = decode_hex(color).unwrap();
                    for x in 0..image.width() {
                        for y in 0..image.height() {
                            let rgba = image.get_pixel(x, y).to_rgba();
                            image.put_pixel(x, y, blend(rgba, [color_parts[0], color_parts[1], color_parts[2]], mutator))
                        }
                    }
                },
                Transform::BlendIconTransform { icon, blend_mode } => {
                    /*
                    let mutator = mutate(*blend_mode);
                    let color_parts = decode_hex(color).unwrap();
                    for x in 0..image.width() {
                        for y in 0..image.height() {
                            let rgba = image.get_pixel(x, y).to_rgba();
                            image.put_pixel(x, y, blend(rgba, [color_parts[0], color_parts[1], color_parts[2]], mutator))
                        }
                    }
                    */
                },
                Transform::ScaleTransform { width, height } => {
                    *image = image.resize_exact(*width, *height, image::imageops::FilterType::Nearest);
                }
                Transform::CropTransform { x1, y1, x2, y2 } => {
                    //*image = image.crop_imm(x1, y1, x2 - x1, y2 - y1)
                }
            }
        }

        let size_id = format!("{}x{}", image.width(), image.height());
        let mut size_map = size_to_images.lock().unwrap();
        let vec = (*size_map).entry(size_id.to_owned()).or_insert(Vec::new());
        vec.push(image);
        sprites_objects.lock().unwrap().insert(sprite_name.to_owned(), SpritesheetEntry {
            size_id: size_id.to_owned(),
            position: u32::try_from(vec.len()).unwrap() - 1
        });
    });

    size_to_images.lock().unwrap().par_iter().for_each(|(size_id, images_list)| {
        zone!("join_sprites");
        let file_path = format!("{}{}_{}.png", file_path, spritesheet_name, size_id);
        let size_data: Vec<&str> = size_id.split("x").collect();
        let base_width = size_data.first().unwrap().to_string().parse::<u32>().unwrap();
        let base_height = size_data.last().unwrap().to_string().parse::<u32>().unwrap();

        let image_count: u32 = u32::try_from(images_list.len()).unwrap();
        let mut final_image = DynamicImage::new_rgba8(base_width * image_count, base_height);

        for idx in 0..image_count {
            zone!("join_sprite");
            let image: &DynamicImage = images_list.get::<usize>(usize::try_from(idx).unwrap()).unwrap();
            let base_x: u32 = base_width * idx;
            for x in 0..image.width() {
                for y in 0..image.height() {
                    final_image.put_pixel(base_x + x, y, image.get_pixel(x, y))
                }
            }
        }
        {
            zone!("write_spritesheet");
            final_image.save(file_path).err();
        }
    });

    let sizes: Vec<String> = size_to_images.lock().unwrap().iter().map(|(k, _v)| k).cloned().collect();

    let returned = Returned {
        sizes: sizes,
        sprites: sprites_objects.lock().unwrap().to_owned(),
        error: error.lock().unwrap().join("\n"),
    };
    Ok(serde_json::to_string::<Returned>(&returned).unwrap())
}

/// Takes in an icon and gives a list of nested icons. Also returns a reference to the provided icon in the list.
fn icon_to_icons(icon: &IconObject) -> Vec<&IconObject> {
    let mut icons: Vec<&IconObject> = Vec::new();
    icons.push(icon);
    for transform in &icon.transform {
        match transform {
            Transform::BlendIconTransform { icon, .. } => {
                let nested = icon_to_icons(icon);
                for icon in nested {
                    icons.push(icon)
                }
            }
            _ => {}
        }
    }
    return icons;
}

/// Given an IconObject, returns a DMI Icon structure and caches it.
fn icon_to_dmi(icon: &IconObject) -> Result<&Icon, String> {
    zone!("icon_to_dmi");
    let icon_path: String = icon.icon_file;
    {
        zone!("check_dmi_exists");
        // scope-in so the lock does not persist during DMI read
        let found_icon = ICON_FILES.lock().unwrap().get(&icon_path);
        if found_icon.is_some() {
            return Ok(found_icon.unwrap());
        }
    }
    let reader = BufReader::new(File::open(&icon_path).unwrap());
    let dmi: Option<Icon>;
    {
        zone!("parse_dmi");
        dmi = Icon::load(reader).ok();
    }
    if dmi.is_none() {
        return Err(format!("Invalid DMI: {}", icon_path));
    }
    {
        zone!("insert_dmi");
        let my_dmi = dmi.unwrap();
        // cache it for later.
        // Ownership is given to the hashmap
        ICON_FILES.lock().unwrap().insert(icon_path,my_dmi);
        return Ok(&my_dmi);
    }
}

fn icon_to_image<'a>(icon: &'a IconObject, sprite_name: &String) -> Result<&'a mut DynamicImage, String> {
    {
        zone!("check_dynamicimage_exists");
        // scope-in so the lock does not persist during DMI read
        let found_icon = ICON_STATES.lock().unwrap().get(&icon.to_icostring());
        if found_icon.is_some() {
            return Ok(*found_icon.unwrap())
        }
    }
    let result = icon_to_dmi(icon);
    if result.is_err()  {
        return Err(result.unwrap_err());
    }
    let dmi = result.unwrap();
    let mut matched_state: Option<&IconState> = Option::None;
    {
        zone!("match_icon_state");
        for icon_state in dmi.states {
            if icon_state.name == icon.icon_state {
                matched_state = Option::Some(&icon_state);
                break;
            }
        }
    }
    if matched_state.is_none() {
        return Err(format!("Could not find associated icon state {} for {}", icon.icon_state, sprite_name));
    }
    let state = matched_state.unwrap();
    {
        zone!("determine_icon_state_validity");
        if state.frames < icon.frame {
            return Err(format!("Could not find associated frame: {} in {} icon_state {} - dirs: {} frames: {}", icon.frame, sprite_name, icon.icon_state, state.dirs, state.frames));
        }
        if (state.dirs == 1 && icon.dir != SOUTH)
        || (state.dirs == 4 && !FOUR_DIRS.contains(&icon.dir))
        || (state.dirs == 8 && !EIGHT_DIRS.contains(&icon.dir)) {
            return Err(format!("Invalid dir {} or size of dirs {} in {} state: {} for sprite {}", icon.dir, state.dirs, icon.icon_file, icon.icon_state, sprite_name));
        }

    }
    let icon_index = DIR_TO_INDEX.get(icon.dir as usize);
    if icon_index.is_none() || *icon_index.unwrap() == 255 {
        return Err(format!("Invalid dir {} or size of dirs {} in {} state: {} for sprite {}", icon.dir, state.dirs, icon.icon_file, icon.icon_state, sprite_name));
    }
    let mut icon_idx: u32 = *icon_index.unwrap() as u32;
    if icon.frame > 1 {
        // Add one so zero scales properly
        icon_idx = (icon_idx + 1) * icon.frame - 1
    }
    let image: &mut DynamicImage = state.images.get_mut(icon_idx as usize).unwrap();
    {
        zone!("insert_dynamicimage");
        // cache it for later.
        // Ownership is given to the hashmap
        ICON_STATES.lock().unwrap().insert(icon.to_icostring(), image);
    }
    return Ok(image);
}

fn mutate(blend_mode: u8) -> fn(u8, u8) -> u8 {
    return match blend_mode {
        0 => {|a: u8, b: u8| cap(a as u32 + b as u32)}
        2 => {|a: u8, b: u8| cap(a as u32 * b as u32)}
        3 => {|a: u8, b: u8| {
            if a < 128 {
                return cap(2 * a as u32 * b as u32);
            } else {
                return cap(255 - 510 * (255 - a as u32) * (255 - b as u32));
            }
        }}
        _ => {|a: u8, _: u8| a}
    };
}

fn blend(rgba_src: Rgba<u8>, rgba_dst: [u8; 3], mutator_rgb: fn(u8, u8) -> u8) -> Rgba<u8> {
    let r = mutator_rgb(rgba_src.0[0], rgba_dst[0]);
    let g = mutator_rgb(rgba_src.0[1], rgba_dst[1]);
    let b = mutator_rgb(rgba_src.0[2], rgba_dst[2]);
    let a = rgba_src.0[3];
    return Rgba::<u8>( [r, g, b, a] )
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (1..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

fn cap(val: u32) -> u8 {
    if val > 255 {
        return 255;
    } else {
        return u8::try_from(val).unwrap();
    }
}
