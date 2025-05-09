// Multi-threaded DMI spritesheet generator and GAGS re-implementation
// Developed by itsmeow
pub mod blending;
pub mod byond;
pub mod image_cache;
pub mod gags;
pub mod icon_operations;
pub mod spritesheet;
use crate::{
    error::Error,
    hash::{file_hash, string_hash},
};
use dashmap::{DashMap, DashSet};
use dmi::icon::{DmiVersion, Icon, IconState};
use image::{DynamicImage, RgbaImage};
use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::RwLock;
use std::{
    collections::HashMap,
    fs::File,
    hash::BuildHasherDefault,
    sync::{Arc, Mutex},
};
use tracy_full::zone;
use twox_hash::XxHash64;

type SpriteJsonMap = HashMap<String, HashMap<String, UniversalIcon>, BuildHasherDefault<XxHash64>>;
/// This is used to save time decoding 'sprites' a second time between the cache step and the generate step.
static SPRITES_TO_JSON: Lazy<Arc<Mutex<SpriteJsonMap>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::with_hasher(BuildHasherDefault::<
        XxHash64,
    >::default())))
});

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct UniversalIcon {
    icon_file: String,
    icon_state: String,
    dir: Option<u8>,
    frame: Option<u32>,
    transform: Vec<Transform>,
}

impl std::fmt::Display for UniversalIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "UniversalIcon(icon_file={}, icon_state={}, dir={:?}, frame={:?})",
            self.icon_file, self.icon_state, self.dir, self.frame
        )
    }
}

impl UniversalIcon {
    fn to_base(&self) -> Result<Self, Error> {
        zone!("to_base");
        Ok(UniversalIcon {
            icon_file: self.icon_file.to_owned(),
            icon_state: self.icon_state.to_owned(),
            dir: self.dir,
            frame: self.frame,
            transform: Vec::new()
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(tag = "type")]
enum Transform {
    BlendColor { color: String, blend_mode: u8 },
    BlendIcon { icon: UniversalIcon, blend_mode: u8 },
    Scale { width: u32, height: u32 },
    Crop { x1: i32, y1: i32, x2: i32, y2: i32 },
}

#[derive(Serialize)]
struct CacheResult {
    result: String,
    fail_reason: String,
}

fn cache_valid(input_hash: &str, dmi_hashes_in: &str, sprites_in: &str) -> Result<String, Error> {
    zone!("cache_valid");
    let sprites_hash = string_hash("xxh64_fixed", sprites_in)?;
    if sprites_hash != input_hash {
        return Ok(serde_json::to_string::<CacheResult>(&CacheResult {
            result: String::from("0"),
            fail_reason: String::from("Input hash did not match."),
        })?);
    }
    let dmi_hashes: DashMap<String, String>;
    {
        zone!("from_json_hashes");
        dmi_hashes = serde_json::from_str::<DashMap<String, String>>(dmi_hashes_in)?;
    }
    let mut sprites_json: std::sync::MutexGuard<
        '_,
        HashMap<String, HashMap<String, UniversalIcon>, BuildHasherDefault<XxHash64>>,
    > = SPRITES_TO_JSON.lock().unwrap();
    let sprites = match sprites_json.get(&sprites_hash) {
        Some(sprites) => sprites,
        None => {
            zone!("from_json_sprites");
            {
                sprites_json.insert(
                    sprites_hash.clone(),
                    serde_json::from_str::<HashMap<String, UniversalIcon>>(sprites_in)?,
                );
            }
            sprites_json.get(&sprites_hash).unwrap()
        }
    };

    let dmis: HashSet<String>;

    {
        zone!("collect_dmis");
        dmis = sprites
            .par_iter()
            .flat_map(|(_, icon)| {
                icon_to_icons(icon)
                    .into_iter()
                    .map(|icon| icon.icon_file.clone())
                    .collect::<HashSet<String>>()
            })
            .collect();
    }

    drop(sprites_json);

    if dmis.len() > dmi_hashes.len() {
        return Ok(serde_json::to_string::<CacheResult>(&CacheResult {
            result: String::from("0"),
            fail_reason: format!("Input hash matched, but more DMIs exist than DMI hashes provided ({} DMIs, {} DMI hashes).", dmis.len(), dmi_hashes.len()),
        })?);
    }

    let fail_reason: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
    {
        zone!("check_dmis");
        dmis.into_par_iter().for_each(|dmi_path| {
            zone!("check_dmi");
            if fail_reason.read().unwrap().is_some() {
                return;
            }
            match dmi_hashes.get(&dmi_path) {
                Some(hash) => {
                    zone!("hash_dmi");
                    match file_hash("xxh64_fixed", &dmi_path) {
                        Ok(new_hash) => {
                            zone!("check_match");
                            if new_hash != *hash {
                                if fail_reason.read().unwrap().is_some() {
                                    return;
                                }
                                *fail_reason.write().unwrap() = Some(format!("Input hash matched, but dmi_hash was invalid DMI: dmi_path (stored hash: {}, new hash: {new_hash})", hash.clone()));
                            }
                        },
                        Err(err) => {
                            if fail_reason.read().unwrap().is_some() {
                                return;
                            }
                            *fail_reason.write().unwrap() = Some(format!("ERROR: Error while hashing dmi_path '{dmi_path}': {err}"));
                        }
                    }
                }
                None => {
                    if fail_reason.read().unwrap().is_some() {
                        return;
                    }
                    *fail_reason.write().unwrap() = Some(format!("Input hash matched, but no dmi_hash existed for DMI: '{dmi_path}'"));
                }
            }
        });
    }
    if let Some(err) = fail_reason.read().unwrap().clone() {
        return Ok(serde_json::to_string::<CacheResult>(&CacheResult {
            result: String::from("0"),
            fail_reason: err,
        })?);
    }
    Ok(serde_json::to_string::<CacheResult>(&CacheResult {
        result: String::from("1"),
        fail_reason: String::from(""),
    })?)
}

fn generate_spritesheet(
    file_path: &str,
    spritesheet_name: &str,
    sprites: &str,
    hash_icons: &str,
    generate_dmi: &str,
) -> std::result::Result<String, Error> {
    zone!("generate_spritesheet");
    let hash_icons: bool = hash_icons == "1";
    let generate_dmi: bool = generate_dmi == "1";
    let error = Arc::new(Mutex::new(Vec::<String>::new()));
    let dmi_hashes = DashMap::<String, String>::new();

    let size_to_icon_objects = Arc::new(Mutex::new(
        HashMap::<String, Vec<(&String, &UniversalIcon)>>::new(),
    ));
    let sprites_objects =
        DashMap::<String, spritesheet::SpritesheetEntry, BuildHasherDefault<XxHash64>>::with_hasher(
            BuildHasherDefault::<XxHash64>::default(),
        );

    let tree_bases = Arc::new(Mutex::new(HashMap::<
        UniversalIcon,
        Vec<(&String, &UniversalIcon)>,
        BuildHasherDefault<XxHash64>,
    >::with_hasher(
        BuildHasherDefault::<XxHash64>::default()
    )));
    let sprites_hash;
    {
        zone!("compute_sprites_hash");
        sprites_hash = string_hash("xxh64_fixed", sprites)?;
    }
    let sprites_map = match SPRITES_TO_JSON.lock().unwrap().get(&sprites_hash) {
        Some(sprites) => sprites.clone(),
        None => {
            zone!("from_json_sprites"); // byondapi, save us
            serde_json::from_str::<HashMap<String, UniversalIcon>>(sprites)?
        }
    };

    // Pre-load all the DMIs now.
    // This is much faster than doing it as we go (tested!), because sometimes multiple parallel iterators need the DMI.
    sprites_map.par_iter().for_each(|(sprite_name, icon)| {
        zone!("sprite_to_icons");

        icon_to_icons(icon).into_par_iter().for_each(|icon| {
            match image_cache::filepath_to_dmi(&icon.icon_file) {
                Ok(_) => {
                    if hash_icons && !dmi_hashes.contains_key(&icon.icon_file) {
                        zone!("hash_dmi");
                        match file_hash("xxh64_fixed", &icon.icon_file) {
                            Ok(hash) => {
                                zone!("insert_dmi_hash");
                                dmi_hashes.insert(icon.icon_file.clone(), hash);
                            }
                            Err(err) => {
                                error.lock().unwrap().push(err.to_string());
                            }
                        };
                    }
                }
                Err(err) => error.lock().unwrap().push(err),
            }
        });

        {
            zone!("map_to_base");
            let base = match icon.to_base() {
                Ok(base) => base,
                Err(err) => {
                    error.lock().unwrap().push(err.to_string());
                    return;
                }
            };
            tree_bases
                .lock()
                .unwrap()
                .entry(base)
                .or_default()
                .push((sprite_name, icon));
        }
    });

    // cache this here so we don't generate the same string 5000 times
    let sprite_name = String::from("N/A, in tree generation stage");

    // Map duplicate transform sets into a tree.
    // This is beneficial in the case where we have the same base image, and the same set of transforms, but change 1 or 2 things at the end.
    // We can greatly reduce the amount of RgbaImages created by first finding these.
    tree_bases
        .lock()
        .unwrap()
        .par_iter()
        .for_each(|(_, icons)| {
            zone!("transform_trees");
            let first_icon = match icons.first() {
                Some((_, icon)) => icon,
                None => {
                    error
                        .lock()
                        .unwrap()
                        .push(String::from("Somehow found no icon for a tree."));
                    return;
                }
            };
            let (base_image, _) = match image_cache::icon_to_image(first_icon, &sprite_name, false, false)
            {
                Ok(image) => image,
                Err(err) => {
                    error.lock().unwrap().push(err);
                    return;
                }
            };
            let mut no_transforms = Option::<&UniversalIcon>::None;
            let unique_icons = DashSet::<&UniversalIcon>::new();
            {
                zone!("map_unique");
                icons.iter().for_each(|(_, icon)| {
                    // This will ensure we only map unique transform sets. This also means each UniversalIcon is guaranteed a unique icon_hash
                    // Since all icons share the same 'base'.
                    // Also check to see if the icon is already cached. If so, we can ignore this transform chain.
                    if !image_cache::image_cache_contains(icon) {
                        unique_icons.insert(icon);
                    }
                    if icon.transform.is_empty() {
                        no_transforms = Some(icon);
                    }
                });
            }
            if let Some(entry) = no_transforms {
                if let Err(err) = image_cache::return_image(base_image.clone(), entry) {
                    error.lock().unwrap().push(err.to_string());
                }
            }
            {
                zone!("transform_all_leaves");
                if let Err(err) = transform_leaves(
                    &unique_icons.into_iter().collect(),
                    base_image,
                    0,
                ) {
                    error.lock().unwrap().push(err);
                }
            }
        });

    // Pick the specific icon states out of the DMI, also generating their transforms, build the spritesheet metadata.
    sprites_map.par_iter().for_each(|sprite_entry| {
        zone!("map_sprite");
        let (sprite_name, icon) = sprite_entry;

        // get RgbaImage, it should already be transformed, so it must be cached.
        let (image, _) = match image_cache::icon_to_image(icon, sprite_name, true, true) {
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
            if let Err(err) = image_cache::return_image(image, icon) {
                error.lock().unwrap().push(err.to_string());
            }
            let icon_position;
            {
                zone!("insert_into_size_map");
                // This scope releases the lock on size_to_icon_objects
                let mut size_map = size_to_icon_objects.lock().unwrap();
                let vec = (*size_map).entry(size_id.to_owned()).or_default();
                icon_position = vec.len() as u32;
                vec.push(sprite_entry);
            }

            {
                zone!("insert_into_sprite_objects");
                sprites_objects.insert(
                    sprite_name.to_owned(),
                    spritesheet::SpritesheetEntry {
                        size_id: size_id.to_owned(),
                        position: icon_position,
                    },
                );
            }
        }
    });

    // all images have been returned now, so continue...
    // Get all the sprites and spew them onto a spritesheet.
    size_to_icon_objects
        .lock()
        .unwrap()
        .par_iter()
        .for_each(|(size_id, sprite_entries)| {
            zone!("join_sprites");
            let file_path = format!(
                "{file_path}{spritesheet_name}_{size_id}.{}",
                if generate_dmi { "dmi" } else { "png" }
            );
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

            if generate_dmi {
                let output_states = match create_dmi_output_states(sprite_entries) {
                    Ok(output_states) => output_states,
                    Err(err) => {
                        error.lock().unwrap().push(err);
                        return;
                    }
                };
                {
                    zone!("spritesheet_dmi_sort_states");
                    // This is important, because it allows the outputted DMI to be used in IconForge's own cache - they will output in the same order between runs.
                    // PNGs don't need these because they're only usable in the UI, but these DMI icons are potentially persistent (they may be used at compile time!!)
                    output_states
                        .lock()
                        .unwrap()
                        .sort_unstable_by(|state1, state2| state1.name.cmp(&state2.name))
                }
                {
                    zone!("write_spritesheet_dmi");
                    {
                        zone!("create_file");
                        let path = std::path::Path::new(&file_path);
                        if let Err(err) = std::fs::create_dir_all(path.parent().unwrap()) {
                            error.lock().unwrap().push(err.to_string());
                            return;
                        };
                        let mut output_file = match File::create(path) {
                            Ok(file) => file,
                            Err(err) => {
                                error.lock().unwrap().push(err.to_string());
                                return;
                            }
                        };
                        {
                            zone!("save_dmi");
                            Icon {
                                version: DmiVersion::default(),
                                width: base_width,
                                height: base_height,
                                states: output_states.lock().unwrap().to_owned(),
                            }
                            .save(&mut output_file)
                            .err();
                        }
                    }
                }
            } else {
                let final_image = match create_png_image(base_width, base_height, sprite_entries) {
                    Ok(image) => image,
                    Err(err) => {
                        error.lock().unwrap().push(err);
                        return;
                    }
                };
                {
                    zone!("write_spritesheet_png");
                    final_image.save(file_path).err();
                }
            }
        });

    let sizes: Vec<String> = size_to_icon_objects
        .lock()
        .unwrap()
        .keys()
        .cloned()
        .collect();

    // Collect the game metadata and any errors.
    let returned = spritesheet::SpritesheetResult {
        sizes,
        sprites: sprites_objects,
        dmi_hashes,
        sprites_hash,
        error: error.lock().unwrap().join("\n"),
    };
    Ok(serde_json::to_string::<spritesheet::SpritesheetResult>(&returned)?)
}

fn create_png_image(
    base_width: u32,
    base_height: u32,
    sprite_entries: &Vec<(&String, &UniversalIcon)>,
) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, String> {
    zone!("create_png_image");
    let mut final_image = RgbaImage::new(base_width * sprite_entries.len() as u32, base_height);
    for (idx, sprite_entry) in sprite_entries.iter().enumerate() {
        zone!("join_sprite_png");
        let (sprite_name, icon) = *sprite_entry;
        let image = match image_cache::icon_to_image(icon, sprite_name, true, true) {
            Ok((image, _)) => image,
            Err(err) => {
                return Err(err);
            }
        };
        let base_x: u32 = base_width * idx as u32;
        for x in 0..image.width() {
            for y in 0..image.height() {
                final_image.put_pixel(base_x + x, y, *image.get_pixel(x, y))
            }
        }
        if let Err(err) = image_cache::return_image(image, icon) {
            return Err(err.to_string());
        }
    }
    Ok(final_image)
}

fn create_dmi_output_states(
    sprite_entries: &Vec<(&String, &UniversalIcon)>,
) -> Result<Arc<Mutex<Vec<IconState>>>, String> {
    zone!("create_dmi_output_states");
    let output_states = Arc::new(Mutex::new(Vec::<IconState>::new()));
    let errors = Mutex::new(Vec::<String>::new());
    sprite_entries.par_iter().for_each(|sprite_entry| {
        zone!("create_output_state_dmi");
        let (sprite_name, icon) = *sprite_entry;
        let image = match image_cache::icon_to_image(icon, sprite_name, true, true) {
            Ok((image, _)) => image,
            Err(err) => {
                errors.lock().unwrap().push(err);
                return;
            }
        };
        let dynamic_image = DynamicImage::ImageRgba8(image.to_owned());
        if let Err(err) = image_cache::return_image(image, icon) {
            errors.lock().unwrap().push(err.to_string());
            return;
        }
        output_states.lock().unwrap().push(IconState {
            name: sprite_name.to_owned(),
            dirs: 1,
            frames: 1,
            delay: Option::None,
            loop_flag: dmi::icon::Looping::Indefinitely,
            rewind: false,
            movement: false,
            unknown_settings: Option::None,
            hotspot: Option::None,
            images: vec![dynamic_image; 1],
        });
    });
    if !errors.lock().unwrap().is_empty() {
        return Err(errors.lock().unwrap().join("\n"));
    }
    Ok(output_states)
}

/// Given an array of 'transform arrays' onto from a shared UniversalIcon base,
/// recursively applies transforms in a tree structure. Maximum transform depth is 128.
fn transform_leaves(icons: &Vec<&UniversalIcon>, image: RgbaImage, depth: u8) -> Result<(), String> {
    zone!("transform_leaf");
    if depth > 128 {
        return Err(String::from(
            "Transform depth exceeded 128. https://www.youtube.com/watch?v=CUjrySBwi5Q",
        ));
    }
    let next_transforms = DashMap::<Transform, Vec<&UniversalIcon>>::new();
    let errors = Mutex::new(Vec::<String>::new());

    {
        zone!("get_next_transforms");
        icons.par_iter().for_each(|icon| {
            zone!("collect_icon_transforms");
            if let Some(transform) = icon.transform.get(depth as usize) {
                next_transforms
                    .entry(transform.clone())
                    .or_default()
                    .push(icon);
            }
        });
    }

    {
        zone!("do_next_transforms");
        next_transforms
            .into_par_iter()
            .for_each(|(transform, mut associated_icons)| {
                let mut altered_image;
                {
                    zone!("clone_image");
                    altered_image = image.clone();
                }
                if let Err(err) = transform_image(&mut altered_image, &transform) {
                    errors.lock().unwrap().push(err);
                }
                {
                    zone!("filter_associated_icons");
                    associated_icons
                        .clone()
                        .into_iter()
                        .enumerate()
                        .for_each(|(idx, icon)| {
                            if icon.transform.len() as u8 == depth + 1
                                && *icon.transform.last().unwrap() == transform
                            {
                                associated_icons.swap_remove(idx);
                                if let Err(err) = image_cache::return_image(altered_image.clone(), icon) {
                                    errors.lock().unwrap().push(err.to_string());
                                }
                            }
                        });
                }
                if let Err(err) = transform_leaves(&associated_icons, altered_image, depth + 1) {
                    errors.lock().unwrap().push(err);
                }
            });
    }

    if !errors.lock().unwrap().is_empty() {
        return Err(errors.lock().unwrap().join("\n"));
    }
    Ok(())
}

/// Takes in an icon and gives a list of nested icons. Also returns a reference to the provided icon in the list.
fn icon_to_icons(icon_in: &UniversalIcon) -> Vec<&UniversalIcon> {
    zone!("icon_to_icons");
    let mut icons: Vec<&UniversalIcon> = Vec::new();
    icons.push(icon_in);
    for transform in &icon_in.transform {
        if let Transform::BlendIcon { icon, .. } = transform {
            let nested = icon_to_icons(icon);
            for icon in nested {
                icons.push(icon)
            }
        }
    }
    icons
}

fn apply_all_transforms(image: &mut RgbaImage, transforms: &Vec<Transform>) -> Result<(), String> {
    let mut errors = Vec::<String>::new();
    for transform in transforms {
        if let Err(error) = transform_image(image, transform) {
            errors.push(error);
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(())
}

/// Applies transforms to a RgbaImage.
fn transform_image(image: &mut RgbaImage, transform: &Transform) -> Result<(), String> {
    zone!("transform_image");
    match transform {
        Transform::BlendColor { color, blend_mode } => {
            icon_operations::blend_color(image, color, &blending::BlendMode::from_u8(blend_mode)?)?
        }
        Transform::BlendIcon { icon, blend_mode } => {
            zone!("blend_icon");
            let (mut other_image, cached) =
                image_cache::icon_to_image(icon, &format!("Transform blend_icon {icon}"), true, false)?;

            if !cached {
                apply_all_transforms(&mut other_image, &icon.transform)?;
            };
            icon_operations::blend_icon(
                image,
                &other_image,
                &blending::BlendMode::from_u8(blend_mode)?,
            )?;
            if let Err(err) = image_cache::return_image(other_image, icon) {
                return Err(err.to_string());
            }
        }
        Transform::Scale { width, height } => {
            zone!("scale");
            icon_operations::scale(image, *width, *height);
        }
        Transform::Crop { x1, y1, x2, y2 } => {
            icon_operations::crop(image, *x1, *y1, *x2, *y2)?;
        }
    }
    Ok(())
}
