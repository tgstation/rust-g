// DMI spritesheet generator
// Developed by itsmeow
use crate::jobs;
use crate::hash::string_hash;
use crate::error::Error;
use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
    collections::HashMap,
};
use dmi::icon::{Icon, IconState};
use image::{DynamicImage, GenericImage, GenericImageView, Pixel, ImageBuffer};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
//use raster::Image;
use serde::{Serialize, Deserialize};
use dashmap::DashMap;
use tracy_full::{zone, frame};
use once_cell::sync::Lazy;
static ICON_FILES: Lazy<DashMap<String, Arc<Icon>>> = Lazy::new(DashMap::new);
static ICON_STATES: Lazy<DashMap<String, DynamicImage>> = Lazy::new(DashMap::new);

const SOUTH: u8 = 2;
const NORTH: u8 = 1;
const EAST: u8 = 4;
const WEST: u8 = 8;
const FOUR_DIRS: [u8; 4] = [SOUTH, NORTH, EAST, WEST];
const SOUTHEAST: u8 = SOUTH | EAST; // 6
const SOUTHWEST: u8 = SOUTH | WEST; // 10
const NORTHEAST: u8 = NORTH | EAST; // 5
const NORTHWEST: u8 = NORTH | WEST; // 9
// This is ordered by how DMIs internally place dirs into the PNG
const EIGHT_DIRS: [u8; 8] = [SOUTH, NORTH, EAST, WEST, SOUTHEAST, SOUTHWEST, NORTHEAST, NORTHWEST];

// This is an array mapping the DIR number from above to a position in DMIs, such that DIR_TO_INDEX[DIR] = EIGHT_DIRS.indexof(DIR)
// 255 is invalid.
const DIR_TO_INDEX: [u8; 11] = [255, 1, 0, 255, 2, 6, 4, 255, 3, 7, 5];


byond_fn!(fn iconforge_generate(file_path, spritesheet_name, sprites) {
    let file_path = file_path.to_owned();
    let spritesheet_name = spritesheet_name.to_owned();
    let sprites = sprites.to_owned();
    Some(match catch_panic(&file_path, &spritesheet_name, &sprites) {
        Ok(o) => o.to_string(),
        Err(e) => e.to_string()
    })
});


byond_fn!(fn iconforge_generate_async(file_path, spritesheet_name, sprites) {
    // Take ownership before passing
    let file_path = file_path.to_owned();
    let spritesheet_name = spritesheet_name.to_owned();
    let sprites = sprites.to_owned();
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

impl IconObject {
    fn to_icostring(&self) -> Result<String, Error> {
        zone!("to_icostring");
        string_hash("xxh64", &serde_json::to_string(self).unwrap())
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum Transform {
    BlendColor {
        color: String,
        blend_mode: u8,
    },
    BlendIcon {
        icon: IconObject,
        blend_mode: u8,
    },
    Scale {
        width: u32,
        height: u32,
    },
    Crop {
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
        result
    });
    if let Err(err) = x {
        let message: Option<String> = err
            .downcast_ref::<&'static str>()
            .map(|payload| payload.to_string())
            .or_else(|| {
                err.downcast_ref::<String>().cloned()
            });
        return Err(Error::IconState(message.unwrap().to_owned()));
    }
    x.ok().unwrap()
}

fn generate_spritesheet(file_path: &str, spritesheet_name: &str, sprites: &str) -> std::result::Result<String, Error> {
    zone!("generate_spritesheet");

    let error: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let size_to_icon_objects: Arc<Mutex<HashMap<String, Vec<&IconObject>>>> = Arc::new(Mutex::new(HashMap::new()));
    let sprites_map: HashMap<String, IconObject> = serde_json::from_str::<HashMap<String, IconObject>>(sprites)?;
    let sprites_objects: Arc<Mutex<HashMap<String, SpritesheetEntry>>> = Arc::new(Mutex::new(HashMap::new()));

    // Pre-load all the DMIs now.
    sprites_map.par_iter().for_each(|sprite_entry| {
        zone!("sprite_to_icons");
        let (_, icon) = sprite_entry;
        icon_to_icons(icon).par_iter().for_each(|icon| {
            if let Err(err) = icon_to_dmi(icon) {
                error.lock().unwrap().push(err);
            }
        });
    });

    // Pick the specific icon states out of the DMI
    sprites_map.par_iter().for_each(|sprite_entry| {
        zone!("map_sprite");
        let (sprite_name, icon) = sprite_entry;

        // get DynamicImage, applying transforms as well
        let image_result = icon_to_image(icon, sprite_name);
        if let Err(err) = image_result {
            error.lock().unwrap().push(err);
            return;
        }
        let image = image_result.unwrap();

        {
            zone!("create_game_metadata");
            // Generate the metadata used by the game
            let size_id = format!("{}x{}", image.width(), image.height());
            return_image(image, icon);
            let mut size_map = size_to_icon_objects.lock().unwrap();
            let vec = (*size_map).entry(size_id.to_owned()).or_insert(Vec::new());
            vec.push(icon);

            sprites_objects.lock().unwrap().insert(sprite_name.to_owned(), SpritesheetEntry {
                size_id: size_id.to_owned(),
                position: u32::try_from(vec.len()).unwrap() - 1
            });
        }
    });

    // all images have been returned now, so continue...

    // Get all the sprites and spew them onto a spritesheet.
    size_to_icon_objects.lock().unwrap().par_iter().for_each(|(size_id, icon_objects)| {
        zone!("join_sprites");
        let file_path = format!("{}{}_{}.png", file_path, spritesheet_name, size_id);
        let size_data: Vec<&str> = size_id.split('x').collect();
        let base_width = size_data.first().unwrap().to_string().parse::<u32>().unwrap();
        let base_height = size_data.last().unwrap().to_string().parse::<u32>().unwrap();

        let image_count: u32 = u32::try_from(icon_objects.len()).unwrap();
        let mut final_image = DynamicImage::new_rgba8(base_width * image_count, base_height);

        for idx in 0..image_count {
            zone!("join_sprite");
            let icon = icon_objects.get::<usize>(usize::try_from(idx).unwrap()).unwrap();
            let image_result = icon_to_image(icon, &"N/A, in final generation stage".to_string());
            if let Err(err) = image_result {
                error.lock().unwrap().push(err);
                continue;
            }
            let image = image_result.unwrap();
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

    let sizes: Vec<String> = size_to_icon_objects.lock().unwrap().iter().map(|(k, _v)| k).cloned().collect();

    // Collect the game metadata and any errors.
    let returned = Returned {
        sizes,
        sprites: sprites_objects.lock().unwrap().to_owned(),
        error: error.lock().unwrap().join("\n"),
    };
    Ok(serde_json::to_string::<Returned>(&returned).unwrap())
}

/// Takes in an icon and gives a list of nested icons. Also returns a reference to the provided icon in the list.
fn icon_to_icons(icon: &IconObject) -> Vec<&IconObject> {
    zone!("icon_to_icons");
    let mut icons: Vec<&IconObject> = Vec::new();
    icons.push(icon);
    for transform in &icon.transform {
        if let Transform::BlendIcon { icon, .. } = transform  {
            let nested = icon_to_icons(icon);
            for icon in nested {
                icons.push(icon)
            }
        }
    }
    icons
}

/// Given an IconObject, returns a DMI Icon structure and caches it.
fn icon_to_dmi(icon: &IconObject) -> Result<Arc<Icon>, String> {
    zone!("icon_to_dmi");
    let icon_path: &String = &icon.icon_file;
    {
        zone!("check_dmi_exists");
        // scope-in so the lock does not persist during DMI read
        let found_icon = ICON_FILES.get(icon_path);
        if let Some(found) = found_icon {
            return Ok(found.clone());
        }
    }
    let icon_file = File::open(icon_path);
    if icon_file.is_err() {
        return Err(format!("No such DMI file: {}", icon_path))
    }
    let reader = BufReader::new(icon_file.unwrap());
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
        let dmi_arc = Arc::new(dmi.unwrap());
        let other_arc = dmi_arc.clone();
        // cache it for later.
        // Ownership is given to the hashmap
        ICON_FILES.insert(icon_path.to_owned(), dmi_arc);
        Ok(other_arc)
    }
}

// Takes an IconObject, gets its DMI, then picks out a DynamicImage for the IconState, as well as transforms the DynamicImage.
// Gives ownership over the image. Please return when you are done <3
fn icon_to_image(icon: &IconObject, sprite_name: &String) -> Result<DynamicImage, String> {
    zone!("icon_to_image");
    {
        zone!("check_dynamicimage_exists");
        // scope-in so the lock does not persist during DMI read
        let found_icon = ICON_STATES.remove(&icon.to_icostring().unwrap());
        if let Some(found) = found_icon {
            return Ok(found.1)
        }
    }
    let dmi = icon_to_dmi(icon)?;
    let mut matched_state: Option<&IconState> = Option::None;
    {
        zone!("match_icon_state");
        for icon_state in &dmi.states {
            if icon_state.name == icon.icon_state {
                matched_state = Option::Some(icon_state);
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
    let image: DynamicImage = state.images.get(icon_idx as usize).unwrap().clone();
    // Apply transforms
    let (transformed_image, errors) = transform_image(image, icon, sprite_name);
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(transformed_image)
}

// Gives an image back to the cache, after it is done being used.
fn return_image(image: DynamicImage, icon: &IconObject) {
    zone!("insert_dynamicimage");
    ICON_STATES.insert(icon.to_icostring().unwrap(), image);
}

// Applies transforms to a DynamicImage.
fn transform_image(image_in: DynamicImage, icon: &IconObject, sprite_name: &String) -> (DynamicImage, String) {
    zone!("transform_image");
    let mut image = image_in;
    let mut error: Vec<String> = Vec::new();
    for transform in &icon.transform {
        match transform {
            Transform::BlendColor { color, blend_mode } => {
                zone!("blend_color");
                let mut hex: String = color.to_owned();
                if hex.starts_with('#') {
                    hex = hex[1..].to_string();
                }
                if hex.len() == 6 {
                    hex = format!("{}ff", hex);
                }
                let mut color2: [u8; 4] = [0, 0, 0, 255];
                hex::decode_to_slice(hex, &mut color2).expect(&format!("Decoding hex color {} failed", color));
                for x in 0..image.width() {
                    for y in 0..image.height() {
                        let px = image.get_pixel(x, y);
                        let pixel = px.channels();
                        let blended = blend(pixel, &color2, *blend_mode);

                        image.put_pixel(x, y,
                            image::Rgba::<u8>(blended),
                        );
                    }
                }
            },
            Transform::BlendIcon { icon, blend_mode } => {
                zone!("blend_icon");
                let image_result = icon_to_image(icon, &format!("Transform blend_icon of {}", sprite_name));
                if let Err(err) = image_result {
                    error.push(err);
                    continue;
                }

                let other_image = image_result.unwrap();

                for x in 0..image.width() {
                    if x >= other_image.width() {
                        break; // undefined behavior in DM :)
                    }
                    for y in 0..image.height() {
                        if y >= other_image.height() {
                            break; // undefined behavior in DM :)
                        }
                        let px1 = image.get_pixel(x, y);
                        let px2 = other_image.get_pixel(x, y);
                        let pixel_1 = px1.channels();
                        let pixel_2 = px2.channels();

                        let blended = blend(pixel_1, pixel_2, *blend_mode);

                        image.put_pixel(x, y,
                            image::Rgba::<u8>(blended),
                        );
                    }
                }
                return_image(other_image, icon);

            },
            Transform::Scale { width, height } => {
                zone!("scale");
                let x_ratio = image.width() as f32 / *width as f32;
                let y_ratio = image.height() as f32 / *height as f32;
                let mut new_image = DynamicImage::new_rgba8(*width, *height);
                for x in 0..*width {
                    for y in 0..*height {
                        let old_x: u32 = ( x as f32 * x_ratio ).floor() as u32;
                        let old_y: u32 = ( y as f32 * y_ratio ).floor() as u32;
                        let pixel = image.get_pixel(old_x, old_y);
                        new_image.put_pixel(x, y, pixel);
                    }
                }
                image = new_image;
            }
            Transform::Crop { x1, y1, x2, y2 } => {
                zone!("crop");
                let i_width = image.width();
                let i_height = image.height();
                let mut x1 = *x1;
                let mut y1 = *y1;
                let mut x2 = *x2;
                let mut y2 = *y2;
                if x2 <= x1 || y2 <= y1 {
                    error.push(format!("Invalid bounds {} {} to {} {} from sprite {}", x1, y1, x2, y2, sprite_name));
                    continue;
                }

                // convert from BYOND (0,0 is bottom left) to Rust (0,0 is top left)
                let y2_old = y2;
                y2 = i_height as i32 - y1;
                y1 = i_height as i32 - y2_old;

                let mut width = x2 - x1;
                let mut height = y2 - y1;

                if x1 < 0 || x2 > i_width as i32 || y1 < 0 || y2 > i_height as i32 {
                    //continue;
                    let mut blank_img = ImageBuffer::from_fn(width as u32, height as u32, |_x, _y| image::Rgba([0, 0, 0, 0]));
                    image::imageops::overlay(
                    &mut blank_img,
                    &image,
                    if x1 < 0 { (x1).abs() as i64 } else { 0 } - if x1 > i_width as i32 { (x1 - i_width as i32) as i64 } else { 0 },
                    if y1 < 0 { (y1).abs() as i64 } else { 0 } - if x1 > i_width as i32 { (x1 - i_width as i32) as i64} else { 0 },
                    );
                    image = DynamicImage::new_rgba8(width as u32, height as u32);
                    let error_i = image.copy_from(&blank_img, 0, 0);
                    if let Err(err) = error_i {
                        error.push(err.to_string());
                        continue;
                    }
                    assert_eq!(image.width(), width as u32);
                    assert_eq!(image.height(), height as u32);
                    if x1 < 0 {
                        x1 = 0;
                    }
                    if x2 > i_width as i32 {
                        x2 = i_width as i32;
                    }
                    if y1 < 0 {
                        y1 = 0;
                    }
                    if y2 > i_height as i32 {
                        y2 = i_height as i32;
                    }
                    width = x2 - x1;
                    height = y2 - y1;
                }
                image = image.crop_imm(x1 as u32, y1 as u32, width as u32, height as u32);
            }
        }
    }
    (image, error.join("\n"))
}

// Blends two colors according to blend_mode. The numbers correspond to BYOND blend modes.
fn blend(color: &[u8], color2: &[u8], blend_mode: u8) -> [u8; 4] {
    match blend_mode {
        0 => [
            strict_f32_to_u8(color2[0] as f32 + color[0] as f32),
            strict_f32_to_u8(color2[1] as f32 + color[1] as f32),
            strict_f32_to_u8(color2[2] as f32 + color[2] as f32),
            if color2[3] > color[3] {color[3]} else {color2[3]}
        ],
        1 => [
            strict_f32_to_u8(color2[0] as f32 - color[0] as f32),
            strict_f32_to_u8(color2[1] as f32 - color[1] as f32),
            strict_f32_to_u8(color2[2] as f32 - color[2] as f32),
            if color2[3] > color[3] {color[3]} else {color2[3]}
        ],
        2 => [
            strict_f32_to_u8((color[0] as f32) * (color2[0] as f32) / 255.0f32),
            strict_f32_to_u8((color[1] as f32) * (color2[1] as f32) / 255.0f32),
            strict_f32_to_u8((color[2] as f32) * (color2[2] as f32) / 255.0f32),
            strict_f32_to_u8((color[3] as f32) * (color2[3] as f32) / 255.0f32)
        ],
        3 => {
            let mut high = color2[3];
            let mut low = color[3];
            if high < low {
                high = color[3];
                low = color2[3];
            }
            [
                strict_f32_to_u8(color[0] as f32 + (color2[0] as f32 - color[0] as f32) * color2[3] as f32  / 255.0f32),
                strict_f32_to_u8(color[1] as f32 + (color2[1] as f32 - color[1] as f32) * color2[3] as f32  / 255.0f32),
                strict_f32_to_u8(color[2] as f32 + (color2[2] as f32 - color[2] as f32) * color2[3] as f32 / 255.0f32),
                strict_f32_to_u8(high as f32 + (high as f32 * low as f32 / 255.0))
            ]
        },
        6 => {
            let mut high = color[3];
            let mut low = color2[3];
            if high < low {
                high = color2[3];
                low = color[3];
            }
            [
                strict_f32_to_u8(color2[0] as f32 + (color[0] as f32 - color2[0] as f32) * color[3] as f32 / 255.0f32),
                strict_f32_to_u8(color2[1] as f32 + (color[1] as f32 - color2[1] as f32) * color[3] as f32 / 255.0f32),
                strict_f32_to_u8(color2[2] as f32 + (color[2] as f32 - color2[2] as f32) * color[3] as f32 / 255.0f32),
                strict_f32_to_u8(high as f32 + (high as f32 * low as f32 / 255.0f32))
            ]
        },
        _ => [color[0], color[1], color[2], color[3]],
    }
}

// caps an f32 into u8 ranges, rounds it to the nearest integer, then truncates to a u8.
fn strict_f32_to_u8(x: f32) -> u8 {
    if x < u8::MIN as f32 {
        return 0;
    }
    if x > u8::MAX as f32 {
        return u8::MAX;
    }
    x.round().trunc() as u8
}
