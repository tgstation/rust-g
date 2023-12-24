// DMI spritesheet generator
// Developed by itsmeow
use crate::error::Error;
use crate::hash::string_hash;
use crate::jobs;
use dashmap::DashMap;
use dmi::icon::{Icon, IconState};
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Pixel};
use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
};
use tracy_full::{frame, zone};
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

/// This is ordered by how DMIs internally place dirs into the PNG
const EIGHT_DIRS: [u8; 8] = [
    SOUTH, NORTH, EAST, WEST, SOUTHEAST, SOUTHWEST, NORTHEAST, NORTHWEST,
];

/// This is an array mapping the DIR number from above to a position in DMIs, such that DIR_TO_INDEX[DIR] = EIGHT_DIRS.indexof(DIR)
/// 255 is invalid.
const DIR_TO_INDEX: [u8; 11] = [255, 1, 0, 255, 2, 6, 4, 255, 3, 7, 5];

byond_fn!(fn iconforge_generate(file_path, spritesheet_name, sprites) {
    let file_path = file_path.to_owned();
    let spritesheet_name = spritesheet_name.to_owned();
    let sprites = sprites.to_owned();
    Some(match generate_spritesheet_safe(&file_path, &spritesheet_name, &sprites) {
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
        match generate_spritesheet_safe(&file_path, &spritesheet_name, &sprites) {
            Ok(o) => o.to_string(),
            Err(e) => e.to_string()
        }
    }))
});

byond_fn!(fn iconforge_check(id) {
    Some(jobs::check(id))
});

#[derive(Serialize)]
struct SpritesheetResult {
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
    transform: Vec<Transform>,
}

impl IconObject {
    fn to_icostring(&self) -> Result<String, Error> {
        zone!("to_icostring");
        string_hash("xxh64", &serde_json::to_string(self)?)
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum Transform {
    BlendColor { color: String, blend_mode: u8 },
    BlendIcon { icon: IconObject, blend_mode: u8 },
    Scale { width: u32, height: u32 },
    Crop { x1: i32, y1: i32, x2: i32, y2: i32 },
}

fn generate_spritesheet_safe(
    file_path: &str,
    spritesheet_name: &str,
    sprites: &str,
) -> std::result::Result<String, Error> {
    match std::panic::catch_unwind(|| {
        let result = generate_spritesheet(file_path, spritesheet_name, sprites);
        frame!();
        result
    }) {
        Ok(o) => o,
        Err(e) => {
            let message: Option<String> = e
                .downcast_ref::<&'static str>()
                .map(|payload| payload.to_string())
                .or_else(|| e.downcast_ref::<String>().cloned());
            Err(Error::IconForge(
                message
                    .unwrap_or("Failed to stringify panic! Check rustg-panic.log".to_string())
                    .to_owned(),
            ))
        }
    }
}

fn generate_spritesheet(
    file_path: &str,
    spritesheet_name: &str,
    sprites: &str,
) -> std::result::Result<String, Error> {
    zone!("generate_spritesheet");

    let error = Arc::new(Mutex::new(Vec::<String>::new()));

    let size_to_icon_objects = Arc::new(Mutex::new(HashMap::<String, Vec<&IconObject>>::new()));
    let sprites_map = serde_json::from_str::<HashMap<String, IconObject>>(sprites)?;
    let sprites_objects = Arc::new(Mutex::new(HashMap::<String, SpritesheetEntry>::new()));

    // Pre-load all the DMIs now.
    // This is much faster than doing it as we go (tested!), because sometimes multiple parallel iterators need the DMI.
    sprites_map.par_iter().for_each(|sprite_entry| {
        zone!("sprite_to_icons");
        let (_, icon) = sprite_entry;
        icon_to_icons(icon).par_iter().for_each(|icon| {
            if let Err(err) = icon_to_dmi(icon) {
                error.lock().unwrap().push(err);
            }
        });
    });

    // Pick the specific icon states out of the DMI, also generating their transforms, build the spritesheet metadata.
    sprites_map.par_iter().for_each(|sprite_entry| {
        zone!("map_sprite");
        let (sprite_name, icon) = sprite_entry;

        // get DynamicImage, applying transforms as well
        let image = match icon_to_image(icon, sprite_name) {
            Ok(image) => image,
            Err(err) => {
                error.lock().unwrap().push(err);
                return;
            }
        };

        {
            zone!("create_game_metadata");
            // Generate the metadata used by the game
            let size_id = format!("{}x{}", image.width(), image.height());
            if let Err(err) = return_image(image, icon) {
                error.lock().unwrap().push(err.to_string());
            }
            let icon_position;
            {
                zone!("insert_into_size_map");
                // This scope releases the lock on size_to_icon_objects
                let mut size_map = size_to_icon_objects.lock().unwrap();
                let vec = (*size_map).entry(size_id.to_owned()).or_default();
                icon_position = vec.len() as u32;
                vec.push(icon);
            }

            {
                zone!("insert_into_sprite_objects");
                sprites_objects.lock().unwrap().insert(
                    sprite_name.to_owned(),
                    SpritesheetEntry {
                        size_id: size_id.to_owned(),
                        position: icon_position,
                    },
                );
            }
        }
    });

    // all images have been returned now, so continue...

    // cache this here so we don't generate the same string 5000 times
    let sprite_name = "N/A, in final generation stage".to_string();

    // Get all the sprites and spew them onto a spritesheet.
    size_to_icon_objects
        .lock()
        .unwrap()
        .par_iter()
        .for_each(|(size_id, icon_objects)| {
            zone!("join_sprites");
            let file_path = format!("{}{}_{}.png", file_path, spritesheet_name, size_id);
            let size_data: Vec<&str> = size_id.split('x').collect();
            let base_width = size_data
                .first()
                .unwrap()
                .to_string()
                .parse::<u32>()
                .unwrap();
            let base_height = size_data
                .last()
                .unwrap()
                .to_string()
                .parse::<u32>()
                .unwrap();

            let mut final_image =
                DynamicImage::new_rgba8(base_width * icon_objects.len() as u32, base_height);

            for (idx, icon) in icon_objects.iter().enumerate() {
                zone!("join_sprite");
                let image = match icon_to_image(icon, &sprite_name) {
                    Ok(image) => image,
                    Err(err) => {
                        error.lock().unwrap().push(err);
                        return;
                    }
                };
                let base_x: u32 = base_width * idx as u32;
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

    let sizes: Vec<String> = size_to_icon_objects
        .lock()
        .unwrap()
        .iter()
        .map(|(k, _v)| k)
        .cloned()
        .collect();

    // Collect the game metadata and any errors.
    let returned = SpritesheetResult {
        sizes,
        sprites: sprites_objects.lock().unwrap().to_owned(),
        error: error.lock().unwrap().join("\n"),
    };
    Ok(serde_json::to_string::<SpritesheetResult>(&returned)?)
}

/// Takes in an icon and gives a list of nested icons. Also returns a reference to the provided icon in the list.
fn icon_to_icons(icon: &IconObject) -> Vec<&IconObject> {
    zone!("icon_to_icons");
    let mut icons: Vec<&IconObject> = Vec::new();
    icons.push(icon);
    for transform in &icon.transform {
        if let Transform::BlendIcon { icon, .. } = transform {
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
    let icon_path = &icon.icon_file;
    {
        zone!("check_dmi_exists");
        if let Some(found) = ICON_FILES.get(icon_path) {
            return Ok(found.clone());
        }
    }
    let icon_file = match File::open(icon_path) {
        Ok(icon_file) => icon_file,
        Err(_) => {
            return Err(format!("No such DMI file: {}", icon_path));
        }
    };
    let reader = BufReader::new(icon_file);
    let dmi: Icon;
    {
        zone!("parse_dmi");
        dmi = match Icon::load(reader) {
            Ok(dmi) => dmi,
            Err(_) => {
                return Err(format!("Invalid DMI: {}", icon_path));
            }
        };
    }
    {
        zone!("insert_dmi");
        let dmi_arc = Arc::new(dmi);
        let other_arc = dmi_arc.clone();
        // Cache it for later, saving future DMI parsing operations, which are very slow.
        ICON_FILES.insert(icon_path.to_owned(), dmi_arc);
        Ok(other_arc)
    }
}

/// Takes an IconObject, gets its DMI, then picks out a DynamicImage for the IconState, as well as transforms the DynamicImage.
/// Gives ownership over the image. Please return when you are done <3 (via return_image)
fn icon_to_image(icon: &IconObject, sprite_name: &String) -> Result<DynamicImage, String> {
    zone!("icon_to_image");
    {
        zone!("check_dynamicimage_exists");
        let ico_string = match icon.to_icostring() {
            Ok(ico_string) => ico_string,
            Err(err) => {
                return Err(err.to_string());
            }
        };
        if let Some((_key, value)) = ICON_STATES.remove(&ico_string) {
            return Ok(value);
        }
    }
    let dmi = icon_to_dmi(icon)?;
    let mut matched_state: Option<&IconState> = None;
    {
        zone!("match_icon_state");
        for icon_state in &dmi.states {
            if icon_state.name == icon.icon_state {
                matched_state = Some(icon_state);
                break;
            }
        }
    }
    let state = match matched_state {
        Some(state) => state,
        None => {
            return Err(format!(
                "Could not find associated icon state {} for {}",
                icon.icon_state, sprite_name
            ));
        }
    };
    {
        zone!("determine_icon_state_validity");
        if state.frames < icon.frame {
            return Err(format!(
                "Could not find associated frame: {} in {} icon_state {} - dirs: {} frames: {}",
                icon.frame, sprite_name, icon.icon_state, state.dirs, state.frames
            ));
        }
        if (state.dirs == 1 && icon.dir != SOUTH)
            || (state.dirs == 4 && !FOUR_DIRS.contains(&icon.dir))
            || (state.dirs == 8 && !EIGHT_DIRS.contains(&icon.dir))
        {
            return Err(format!(
                "Invalid dir {} or size of dirs {} in {} state: {} for sprite {}",
                icon.dir, state.dirs, icon.icon_file, icon.icon_state, sprite_name
            ));
        }
    }
    let mut icon_idx = match DIR_TO_INDEX.get(icon.dir as usize) {
        Some(idx) if *idx == 255 => {
            return Err(format!(
                "Invalid dir {} or size of dirs {} in {} state: {} for sprite {}",
                icon.dir, state.dirs, icon.icon_file, icon.icon_state, sprite_name
            ));
        }
        Some(idx) => *idx as u32,
        None => {
            return Err(format!(
                "Invalid dir {} or size of dirs {} in {} state: {} for sprite {}",
                icon.dir, state.dirs, icon.icon_file, icon.icon_state, sprite_name
            ));
        }
    };
    if icon.frame > 1 {
        // Add one so zero scales properly
        icon_idx = (icon_idx + 1) * icon.frame - 1
    }
    let image = match state.images.get(icon_idx as usize) {
        Some(image) => image.clone(),
        None => {
            return Err(
                format!("Out of bounds index {} in icon_state {} for sprite {} - Maximum index: {} (frames: {}, dirs: {})",
                icon_idx, icon.icon_state, sprite_name, state.images.len(), state.dirs, state.frames
            ));
        }
    };
    // Apply transforms
    let (transformed_image, errors) = transform_image(image, icon, sprite_name);
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(transformed_image)
}

/// Gives an image back to the cache, after it is done being used.
fn return_image(image: DynamicImage, icon: &IconObject) -> Result<(), Error> {
    zone!("insert_dynamicimage");
    ICON_STATES.insert(icon.to_icostring()?, image);
    Ok(())
}

/// Applies transforms to a DynamicImage.
fn transform_image(
    image_in: DynamicImage,
    icon: &IconObject,
    sprite_name: &String,
) -> (DynamicImage, String) {
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
                if let Err(err) = hex::decode_to_slice(hex, &mut color2) {
                    error.push(format!("Decoding hex color {} failed: {}", color, err));
                }
                for x in 0..image.width() {
                    for y in 0..image.height() {
                        let px = image.get_pixel(x, y);
                        let pixel = px.channels();
                        let blended = blend_u8(pixel, &color2, *blend_mode);

                        image.put_pixel(x, y, image::Rgba::<u8>(blended));
                    }
                }
            }
            Transform::BlendIcon { icon, blend_mode } => {
                zone!("blend_icon");
                let other_image = match icon_to_image(
                    icon,
                    &format!("Transform blend_icon of {}", sprite_name),
                ) {
                    Ok(other_image) => other_image,
                    Err(err) => {
                        error.push(err);
                        continue;
                    }
                };

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

                        let blended = blend_u8(pixel_1, pixel_2, *blend_mode);

                        image.put_pixel(x, y, image::Rgba::<u8>(blended));
                    }
                }
                if let Err(err) = return_image(other_image, icon) {
                    error.push(err.to_string());
                }
            }
            Transform::Scale { width, height } => {
                zone!("scale");
                let x_ratio = image.width() as f32 / *width as f32;
                let y_ratio = image.height() as f32 / *height as f32;
                let mut new_image = DynamicImage::new_rgba8(*width, *height);
                for x in 0..*width {
                    for y in 0..*height {
                        let old_x: u32 = (x as f32 * x_ratio).floor() as u32;
                        let old_y: u32 = (y as f32 * y_ratio).floor() as u32;
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
                    error.push(format!(
                        "Invalid bounds {} {} to {} {} from sprite {}",
                        x1, y1, x2, y2, sprite_name
                    ));
                    continue;
                }

                // convert from BYOND (0,0 is bottom left) to Rust (0,0 is top left)
                let y2_old = y2;
                y2 = i_height as i32 - y1;
                y1 = i_height as i32 - y2_old;

                let mut width = x2 - x1;
                let mut height = y2 - y1;

                if x1 < 0 || x2 > i_width as i32 || y1 < 0 || y2 > i_height as i32 {
                    let mut blank_img =
                        ImageBuffer::from_fn(width as u32, height as u32, |_x, _y| {
                            image::Rgba([0, 0, 0, 0])
                        });
                    image::imageops::overlay(
                        &mut blank_img,
                        &image,
                        if x1 < 0 { (x1).abs() as i64 } else { 0 }
                            - if x1 > i_width as i32 {
                                (x1 - i_width as i32) as i64
                            } else {
                                0
                            },
                        if y1 < 0 { (y1).abs() as i64 } else { 0 }
                            - if x1 > i_width as i32 {
                                (x1 - i_width as i32) as i64
                            } else {
                                0
                            },
                    );
                    image = DynamicImage::new_rgba8(width as u32, height as u32);
                    if let Err(err) = image.copy_from(&blank_img, 0, 0) {
                        error.push(err.to_string());
                        continue;
                    }
                    x1 = std::cmp::max(0, x1);
                    x2 = std::cmp::min(i_width as i32, x2);
                    y1 = std::cmp::max(0, y1);
                    y2 = std::cmp::min(i_height as i32, y2);
                    width = x2 - x1;
                    height = y2 - y1;
                }
                image = image.crop_imm(x1 as u32, y1 as u32, width as u32, height as u32);
            }
        }
    }
    (image, error.join("\n"))
}

struct Rgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Rgba {
    fn into_array(self) -> [u8; 4] {
        [
            self.r.round() as u8,
            self.g.round() as u8,
            self.b.round() as u8,
            self.a.round() as u8,
        ]
    }

    fn from_array(rgba: &[u8]) -> Rgba {
        Rgba {
            r: rgba[0] as f32,
            g: rgba[1] as f32,
            b: rgba[2] as f32,
            a: rgba[3] as f32,
        }
    }

    fn map_each(
        color: Rgba,
        color2: Rgba,
        rgb_fn: &dyn Fn(f32, f32) -> f32,
        a_fn: &dyn Fn(f32, f32) -> f32,
    ) -> Rgba {
        Rgba {
            r: rgb_fn(color.r, color2.r),
            g: rgb_fn(color.g, color2.g),
            b: rgb_fn(color.b, color2.b),
            a: a_fn(color.a, color2.a),
        }
    }

    fn map_each_a(
        color: Rgba,
        color2: Rgba,
        rgb_fn: &dyn Fn(f32, f32, f32, f32) -> f32,
        a_fn: &dyn Fn(f32, f32) -> f32,
    ) -> Rgba {
        Rgba {
            r: rgb_fn(color.r, color2.r, color.a, color2.a),
            g: rgb_fn(color.g, color2.g, color.a, color2.a),
            b: rgb_fn(color.b, color2.b, color.a, color2.a),
            a: a_fn(color.a, color2.a),
        }
    }
}

fn blend_u8(color: &[u8], color2: &[u8], blend_mode: u8) -> [u8; 4] {
    blend(
        Rgba::from_array(color),
        Rgba::from_array(color2),
        blend_mode,
    )
    .into_array()
}

/// Blends two colors according to blend_mode. The numbers correspond to BYOND blend modes.
fn blend(color: Rgba, color2: Rgba, blend_mode: u8) -> Rgba {
    match blend_mode {
        0 => Rgba::map_each(color, color2, &|c1, c2| c1 + c2, &f32::min),
        1 => Rgba::map_each(color, color2, &|c1, c2| c2 - c1, &f32::min),
        2 => Rgba::map_each(color, color2, &|c1, c2| c1 * c2 / 255.0, &|a1, a2| {
            a1 * a2 / 255.0
        }),
        3 => Rgba::map_each_a(
            color,
            color2,
            &|c1, c2, _c1_a, c2_a| c1 + (c2 - c1) * c2_a / 255.0,
            &|a1, a2| {
                let high = f32::max(a1, a2);
                let low = f32::min(a1, a2);
                high + (high * low / 255.0)
            },
        ),
        6 => Rgba::map_each_a(
            color2,
            color,
            &|c1, c2, _c1_a, c2_a| c1 + (c2 - c1) * c2_a / 255.0,
            &|a1, a2| {
                let high = f32::max(a1, a2);
                let low = f32::min(a1, a2);
                high + (high * low / 255.0)
            },
        ),
        _ => color,
    }
}
