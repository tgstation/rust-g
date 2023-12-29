// DMI spritesheet generator
// Developed by itsmeow
use crate::{
    byond::catch_panic,
    error::Error,
    hash::{file_hash, string_hash},
    jobs,
};
use dashmap::DashMap;
use dmi::{
    dirs::Dirs,
    icon::{Icon, IconState},
};
use image::{Pixel, RgbaImage};
use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::RwLock;
use std::{
    collections::HashMap,
    fs::File,
    hash::BuildHasherDefault,
    io::BufReader,
    sync::{Arc, Mutex},
};
use tracy_full::{frame, zone};
use twox_hash::XxHash64;
type SpriteJsonMap = HashMap<String, HashMap<String, IconObjectIO>, BuildHasherDefault<XxHash64>>;
/// This is used to save time decoding 'sprites' a second time between the cache step and the generate step.
static SPRITES_TO_JSON: Lazy<Arc<Mutex<SpriteJsonMap>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::with_hasher(BuildHasherDefault::<
        XxHash64,
    >::default())))
});
/// A cache of DMI filepath -> Icon objects.
static ICON_FILES: Lazy<DashMap<String, Arc<Icon>, BuildHasherDefault<XxHash64>>> =
    Lazy::new(|| DashMap::with_hasher(BuildHasherDefault::<XxHash64>::default()));
/// A cache of icon_hash_input to RgbaImage (with transforms applied! This can only contain COMPLETED sprites).
static ICON_STATES: Lazy<DashMap<String, RgbaImage, BuildHasherDefault<XxHash64>>> =
    Lazy::new(|| DashMap::with_hasher(BuildHasherDefault::<XxHash64>::default()));

byond_fn!(fn iconforge_generate(file_path, spritesheet_name, sprites, hash_icons) {
    let file_path = file_path.to_owned();
    let spritesheet_name = spritesheet_name.to_owned();
    let sprites = sprites.to_owned();
    let hash_icons = hash_icons.to_owned();
    let result = Some(match catch_panic(|| generate_spritesheet(&file_path, &spritesheet_name, &sprites, &hash_icons)) {
        Ok(o) => o.to_string(),
        Err(e) => e.to_string()
    });
    frame!();
    result
});

byond_fn!(fn iconforge_generate_async(file_path, spritesheet_name, sprites, hash_icons) {
    let file_path = file_path.to_owned();
    let spritesheet_name = spritesheet_name.to_owned();
    let sprites = sprites.to_owned();
    let hash_icons = hash_icons.to_owned();
    Some(jobs::start(move || {
        let result = match catch_panic(|| generate_spritesheet(&file_path, &spritesheet_name, &sprites, &hash_icons)) {
            Ok(o) => o.to_string(),
            Err(e) => e.to_string()
        };
        frame!();
        result
    }))
});

byond_fn!(fn iconforge_check(id) {
    Some(jobs::check(id))
});

byond_fn!(
    fn iconforge_cleanup() {
        ICON_FILES.clear();
        ICON_STATES.clear();
        Some("Ok")
    }
);

byond_fn!(fn iconforge_cache_valid(input_hash, dmi_hashes, sprites) {
    let input_hash = input_hash.to_owned();
    let dmi_hashes = dmi_hashes.to_owned();
    let sprites = sprites.to_owned();
    let result = Some(match catch_panic(|| cache_valid(&input_hash, &dmi_hashes, &sprites)) {
        Ok(o) => o.to_string(),
        Err(e) => e.to_string()
    });
    frame!();
    result
});

byond_fn!(fn iconforge_cache_valid_async(input_hash, dmi_hashes, sprites) {
    let input_hash = input_hash.to_owned();
    let dmi_hashes = dmi_hashes.to_owned();
    let sprites = sprites.to_owned();
    let result = Some(jobs::start(move || {
        match catch_panic(|| cache_valid(&input_hash, &dmi_hashes, &sprites)) {
            Ok(o) => o.to_string(),
            Err(e) => e.to_string()
        }
    }));
    frame!();
    result
});

#[derive(Serialize)]
struct SpritesheetResult {
    sizes: Vec<String>,
    sprites: DashMap<String, SpritesheetEntry, BuildHasherDefault<XxHash64>>,
    dmi_hashes: DashMap<String, String>,
    sprites_hash: String,
    error: String,
}

#[derive(Serialize, Clone)]
struct SpritesheetEntry {
    size_id: String,
    position: u32,
}

#[derive(Serialize, Clone, Eq, PartialEq, Hash)]
struct IconObject {
    icon_file: String,
    icon_state: String,
    dir: u8,
    frame: u32,
    transform: Vec<Transform>,
    transform_hash_input: String,
    icon_hash_input: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct IconObjectIO {
    icon_file: String,
    icon_state: String,
    dir: u8,
    frame: u32,
    transform: Vec<TransformIO>,
}

impl std::fmt::Display for IconObject {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "IconObject(icon_file={}, icon_state={}, dir={}, frame={})",
            self.icon_file, self.icon_state, self.dir, self.frame
        )
    }
}

impl IconObject {
    fn to_base(&self) -> Result<String, Error> {
        zone!("to_base");
        // This is a micro-op that ends up saving a lot of time. format!() is quite slow when you get down to microseconds.
        let mut str_buf = String::with_capacity(self.icon_file.len() + self.icon_state.len() + 4);
        str_buf.push_str(&self.icon_file);
        str_buf.push_str(&self.icon_state);
        str_buf.push_str(&self.dir.to_string());
        str_buf.push_str(&self.frame.to_string());
        Ok(str_buf)
    }

    fn gen_icon_hash_input(&mut self) -> Result<(), Error> {
        zone!("gen_icon_hash_input");
        let base = self.to_base()?;
        {
            zone!("transform_to_json");
            let transform_str = serde_json::to_string(&self.transform)?;
            self.transform_hash_input = transform_str;
        }
        let mut str_buf = String::with_capacity(base.len() + self.transform_hash_input.len());
        str_buf.push_str(&base);
        str_buf.push_str(&self.transform_hash_input);
        self.icon_hash_input = str_buf;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum TransformIO {
    BlendColor { color: String, blend_mode: u8 },
    BlendIcon { icon: IconObjectIO, blend_mode: u8 },
    Scale { width: u32, height: u32 },
    Crop { x1: i32, y1: i32, x2: i32, y2: i32 },
}

#[derive(Serialize, Clone, Eq, PartialEq, Hash)]
enum Transform {
    BlendColor { color: String, blend_mode: u8 },
    BlendIcon { icon: IconObject, blend_mode: u8 },
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
    let mut sprites_json = SPRITES_TO_JSON.lock().unwrap();
    let sprites = match sprites_json.get(&sprites_hash) {
        Some(sprites) => sprites,
        None => {
            zone!("from_json_sprites");
            {
                sprites_json.insert(
                    sprites_hash.clone(),
                    serde_json::from_str::<HashMap<String, IconObjectIO>>(sprites_in)?,
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
                icon_to_icons_io(icon)
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
                                *fail_reason.write().unwrap() = Some(format!("Input hash matched, but dmi_hash was invalid DMI: '{}' (stored hash: {}, new hash: {})", dmi_path, hash.clone(), new_hash));
                            }
                        },
                        Err(err) => {
                            if fail_reason.read().unwrap().is_some() {
                                return;
                            }
                            *fail_reason.write().unwrap() = Some(format!("ERROR: Error while hashing dmi_path '{}': {}", dmi_path, err));
                        }
                    }
                }
                None => {
                    if fail_reason.read().unwrap().is_some() {
                        return;
                    }
                    *fail_reason.write().unwrap() = Some(format!("Input hash matched, but no dmi_hash existed for DMI: '{}'", dmi_path));
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
) -> std::result::Result<String, Error> {
    zone!("generate_spritesheet");
    let hash_icons: bool = hash_icons == "1";
    let error = Arc::new(Mutex::new(Vec::<String>::new()));
    let dmi_hashes = DashMap::<String, String>::new();

    let size_to_icon_objects = Arc::new(Mutex::new(HashMap::<String, Vec<&IconObject>>::new()));
    let sprites_objects =
        DashMap::<String, SpritesheetEntry, BuildHasherDefault<XxHash64>>::with_hasher(
            BuildHasherDefault::<XxHash64>::default(),
        );

    let tree_bases = Arc::new(Mutex::new(HashMap::<
        String,
        Vec<(&String, &IconObject)>,
        BuildHasherDefault<XxHash64>,
    >::with_hasher(
        BuildHasherDefault::<XxHash64>::default()
    )));
    let sprites_hash;
    {
        zone!("compute_sprites_hash");
        sprites_hash = string_hash("xxh64_fixed", sprites)?;
    }
    let input = match SPRITES_TO_JSON.lock().unwrap().get(&sprites_hash) {
        Some(sprites) => sprites.clone(),
        None => {
            zone!("from_json_sprites"); // byondapi, save us
            serde_json::from_str::<HashMap<String, IconObjectIO>>(sprites)?
        }
    };
    let mut sprites_map = HashMap::<String, IconObject>::new();
    {
        zone!("io_to_mem");
        sprites_map.extend(
            input
                .into_par_iter()
                .map(|(sprite_name, icon)| (sprite_name, icon_from_io(icon)))
                .collect::<Vec<(String, IconObject)>>(),
        );
    }

    // Pre-load all the DMIs now.
    // This is much faster than doing it as we go (tested!), because sometimes multiple parallel iterators need the DMI.
    sprites_map.par_iter().for_each(|(sprite_name, icon)| {
        zone!("sprite_to_icons");

        icon_to_icons(icon)
            .into_par_iter()
            .for_each(|icon| match icon_to_dmi(icon) {
                Ok(_) => {
                    if hash_icons {
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
            let (base_image, _) = match icon_to_image(first_icon, &sprite_name, false, false) {
                Ok(image) => image,
                Err(err) => {
                    error.lock().unwrap().push(err);
                    return;
                }
            };
            let mut no_transforms = Option::<&IconObject>::None;
            let unique_icons = DashMap::<String, &IconObject>::new();
            {
                zone!("map_unique");
                icons.iter().for_each(|(_, icon)| {
                    // This will ensure we only map unique transform sets. This also means each IconObject is guaranteed a unique icon_hash
                    // Since all icons share the same 'base'.
                    // Also check to see if the icon is already cached. If so, we can ignore this transform chain.
                    if !ICON_STATES.contains_key(&icon.icon_hash_input) {
                        unique_icons.insert(icon.icon_hash_input.clone(), icon);
                    }
                    if icon.transform.is_empty() {
                        no_transforms = Some(icon);
                    }
                });
            }
            if let Some(entry) = no_transforms {
                if let Err(err) = return_image(base_image.clone(), entry) {
                    error.lock().unwrap().push(err.to_string());
                }
            }
            {
                zone!("transform_all_leaves");
                if let Err(err) = transform_leaves(
                    &unique_icons.into_iter().map(|(_, v)| v).collect(),
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
        let (image, _) = match icon_to_image(icon, sprite_name, true, true) {
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
                sprites_objects.insert(
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
    let sprite_name = String::from("N/A, in final generation stage");

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
                RgbaImage::new(base_width * icon_objects.len() as u32, base_height);

            for (idx, icon) in icon_objects.iter().enumerate() {
                zone!("join_sprite");
                let image = match icon_to_image(icon, &sprite_name, true, true) {
                    Ok((image, _)) => image,
                    Err(err) => {
                        error.lock().unwrap().push(err);
                        return;
                    }
                };
                let base_x: u32 = base_width * idx as u32;
                for x in 0..image.width() {
                    for y in 0..image.height() {
                        final_image.put_pixel(base_x + x, y, *image.get_pixel(x, y))
                    }
                }
                if let Err(err) = return_image(image, icon) {
                    error.lock().unwrap().push(err.to_string());
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
        sprites: sprites_objects,
        dmi_hashes,
        sprites_hash,
        error: error.lock().unwrap().join("\n"),
    };
    Ok(serde_json::to_string::<SpritesheetResult>(&returned)?)
}

/// Given an array of 'transform arrays' onto from a shared IconObject base,
/// recursively applies transforms in a tree structure. Maximum transform depth is 128.
fn transform_leaves(icons: &Vec<&IconObject>, image: RgbaImage, depth: u8) -> Result<(), String> {
    zone!("transform_leaf");
    if depth > 128 {
        return Err(String::from(
            "Transform depth exceeded 128. https://www.youtube.com/watch?v=CUjrySBwi5Q",
        ));
    }
    let next_transforms = DashMap::<Transform, Vec<&IconObject>>::new();
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
                                if let Err(err) = return_image(altered_image.clone(), icon) {
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

/// Converts an IO icon to one with icon_hash_input
fn icon_from_io(icon_in: IconObjectIO) -> IconObject {
    zone!("icon_from_io");
    // TODO: can probably convert this function to just lazily attaching icostring to a RefCell<> or something
    // This alternative type system is too verbose and wasteful of processing time.
    // https://doc.rust-lang.org/reference/interior-mutability.html
    let mut result = IconObject {
        icon_file: icon_in.icon_file,
        icon_state: icon_in.icon_state,
        dir: icon_in.dir,
        frame: icon_in.frame,
        transform: icon_in
            .transform
            .into_iter()
            .map(|transform_in| match transform_in {
                TransformIO::BlendColor { color, blend_mode } => {
                    Transform::BlendColor { color, blend_mode }
                }
                TransformIO::BlendIcon { icon, blend_mode } => Transform::BlendIcon {
                    icon: icon_from_io(icon),
                    blend_mode,
                },
                TransformIO::Crop { x1, y1, x2, y2 } => Transform::Crop { x1, y1, x2, y2 },
                TransformIO::Scale { width, height } => Transform::Scale { width, height },
            })
            .collect(),
        transform_hash_input: String::new(),
        icon_hash_input: String::new(),
    };
    result.gen_icon_hash_input().unwrap(); // unsafe but idc
    result
}

/// Takes in an icon and gives a list of nested icons. Also returns a reference to the provided icon in the list.
fn icon_to_icons(icon_in: &IconObject) -> Vec<&IconObject> {
    zone!("icon_to_icons");
    let mut icons: Vec<&IconObject> = Vec::new();
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

/// icon_to_icons but for IO icons.
fn icon_to_icons_io(icon_in: &IconObjectIO) -> Vec<&IconObjectIO> {
    zone!("icon_to_icons_io");
    let mut icons: Vec<&IconObjectIO> = Vec::new();
    icons.push(icon_in);
    for transform in &icon_in.transform {
        if let TransformIO::BlendIcon { icon, .. } = transform {
            let nested = icon_to_icons_io(icon);
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
        Err(err) => {
            return Err(format!("Failed to open DMI '{}' - {}", icon_path, err));
        }
    };
    let reader = BufReader::new(icon_file);
    let dmi: Icon;
    {
        zone!("parse_dmi");
        dmi = match Icon::load(reader) {
            Ok(dmi) => dmi,
            Err(err) => {
                return Err(format!("DMI '{}' failed to parse - {}", icon_path, err));
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

/// Takes an IconObject, gets its DMI, then picks out a RgbaImage for the IconState.
/// Returns with True if the RgbaImage is pre-cached (and shouldn't have new transforms applied)
/// Gives ownership over the image. Please return when you are done <3 (via return_image)
fn icon_to_image(
    icon: &IconObject,
    sprite_name: &String,
    cached: bool,
    must_be_cached: bool,
) -> Result<(RgbaImage, bool), String> {
    zone!("icon_to_image");
    if cached {
        zone!("check_rgba_image_exists");
        if icon.icon_hash_input.is_empty() {
            return Err(format!(
                "No icon_hash generated for {} {}",
                icon, sprite_name
            ));
        }
        if let Some(entry) = ICON_STATES.get(&icon.icon_hash_input) {
            return Ok((entry.value().clone(), true));
        }
        if must_be_cached {
            return Err(String::from("Image not found in cache!"));
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

    let dir = match Dirs::from_bits(icon.dir) {
        Some(dir) => dir,
        None => {
            return Err(format!(
                "Invalid dir number {} for {}",
                icon.dir, sprite_name
            ));
        }
    };
    Ok(match state.get_image(&dir, icon.frame) {
        Ok(image) => (image.to_rgba8(), false),
        Err(err) => {
            return Err(format!("Error getting image for {}: {}", sprite_name, err));
        }
    })
}

/// Gives an image back to the cache, after it is done being used.
fn return_image(image: RgbaImage, icon: &IconObject) -> Result<(), Error> {
    zone!("insert_rgba_image");
    if icon.icon_hash_input.is_empty() {
        return Err(Error::IconForge(format!(
            "No icon_hash_input generated for {}",
            icon
        )));
    }
    ICON_STATES.insert(icon.icon_hash_input.to_owned(), image);
    Ok(())
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
            zone!("blend_color");
            let mut color2: [u8; 4] = [0, 0, 0, 255];
            {
                zone!("from_hex");
                let mut hex: String = color.to_owned();
                if hex.starts_with('#') {
                    hex = hex[1..].to_string();
                }
                if hex.len() == 6 {
                    hex += "ff";
                }

                if let Err(err) = hex::decode_to_slice(hex, &mut color2) {
                    return Err(format!("Decoding hex color {} failed: {}", color, err));
                }
            }
            for x in 0..image.width() {
                for y in 0..image.height() {
                    let px = image.get_pixel_mut(x, y);
                    let pixel = px.channels();
                    let blended = Rgba::blend_u8(pixel, &color2, *blend_mode);

                    *px = image::Rgba::<u8>(blended);
                }
            }
        }
        Transform::BlendIcon { icon, blend_mode } => {
            zone!("blend_icon");
            let (mut other_image, cached) =
                icon_to_image(icon, &format!("Transform blend_icon {}", icon), true, false)?;

            if !cached {
                apply_all_transforms(&mut other_image, &icon.transform)?;
            };
            for x in 0..std::cmp::min(image.width(), other_image.width()) {
                for y in 0..std::cmp::min(image.width(), other_image.width()) {
                    let px1 = image.get_pixel_mut(x, y);
                    let px2 = other_image.get_pixel(x, y);
                    let pixel_1 = px1.channels();
                    let pixel_2 = px2.channels();

                    let blended = Rgba::blend_u8(pixel_1, pixel_2, *blend_mode);

                    *px1 = image::Rgba::<u8>(blended);
                }
            }
            if let Err(err) = return_image(other_image, icon) {
                return Err(err.to_string());
            }
        }
        Transform::Scale { width, height } => {
            zone!("scale");
            let old_width = image.width() as usize;
            let old_height = image.height() as usize;
            let x_ratio = old_width as f32 / *width as f32;
            let y_ratio = old_height as f32 / *height as f32;
            let mut new_image = RgbaImage::new(*width, *height);
            for x in 0..(*width) {
                for y in 0..(*height) {
                    let old_x = (x as f32 * x_ratio).floor() as u32;
                    let old_y = (y as f32 * y_ratio).floor() as u32;
                    new_image.put_pixel(x, y, *image.get_pixel(old_x, old_y));
                }
            }
            *image = new_image;
        }
        Transform::Crop { x1, y1, x2, y2 } => {
            zone!("crop");

            let i_width = image.width();
            let i_height = image.height();
            let mut x1 = *x1;
            let mut y1 = *y1;
            let mut x2 = *x2;
            let mut y2 = *y2;
            // BYOND indexes from 1,1! how silly of them. We'll just fix this here.
            // Crop(1,1,1,1) is a valid statement. Save us.
            y1 -= 1;
            x1 -= 1;
            if x2 <= x1 || y2 <= y1 {
                return Err(format!(
                    "Invalid bounds {} {} to {} {} in crop transform",
                    x1, y1, x2, y2
                ));
            }

            // Convert from BYOND (0,0 is bottom left) to Rust (0,0 is top left)
            // BYOND also includes the upper bound
            let y2_old = y2;
            y2 = i_height as i32 - y1;
            y1 = i_height as i32 - y2_old;

            // Check for silly expansion crops and add transparency in the gaps.
            if x1 < 0 || x2 > i_width as i32 || y1 < 0 || y2 > i_height as i32 {
                // The amount the blank icon's size should increase by.
                let mut width_inc: u32 = (x2 - i_width as i32).max(0) as u32;
                let mut height_inc: u32 = (y2 - i_height as i32).max(0) as u32;
                // Where to position the icon within our blank space.
                let mut x_offset: u32 = 0;
                let mut y_offset: u32 = 0;
                // Make room to place the image further in, and change our bounds to match.
                if x1 < 0 {
                    x2 += x1.abs();
                    x_offset += x1.unsigned_abs();
                    width_inc += x1.unsigned_abs();
                    x1 = 0;
                }
                if y1 < 0 {
                    y2 += y1.abs();
                    y_offset += y1.unsigned_abs();
                    height_inc += y1.unsigned_abs();
                    y1 = 0;
                }
                let mut blank_img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
                    RgbaImage::from_fn(i_width + width_inc, i_height + height_inc, |_x, _y| {
                        image::Rgba([0, 0, 0, 0])
                    });

                image::imageops::overlay(&mut blank_img, image, x_offset as i64, y_offset as i64);
                *image = image::imageops::crop_imm(
                    &blank_img,
                    x1 as u32,
                    y1 as u32,
                    (x2 - x1) as u32,
                    (y2 - y1) as u32,
                )
                .to_image();
            } else {
                // Normal bounds crop. Hooray!
                *image = image::imageops::crop_imm(
                    image,
                    x1 as u32,
                    y1 as u32,
                    (x2 - x1) as u32,
                    (y2 - y1) as u32,
                )
                .to_image();
            }
        }
    }
    Ok(())
}

#[derive(Clone)]
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
        Self {
            r: rgba[0] as f32,
            g: rgba[1] as f32,
            b: rgba[2] as f32,
            a: rgba[3] as f32,
        }
    }

    fn map_each<F, T>(color: &Rgba, color2: &Rgba, rgb_fn: F, a_fn: T) -> Rgba
    where
        F: Fn(f32, f32) -> f32,
        T: Fn(f32, f32) -> f32,
    {
        Rgba {
            r: rgb_fn(color.r, color2.r),
            g: rgb_fn(color.g, color2.g),
            b: rgb_fn(color.b, color2.b),
            a: a_fn(color.a, color2.a),
        }
    }

    fn map_each_a<F, T>(color: &Rgba, color2: &Rgba, rgb_fn: F, a_fn: T) -> Rgba
    where
        F: Fn(f32, f32, f32, f32) -> f32,
        T: Fn(f32, f32) -> f32,
    {
        Rgba {
            r: rgb_fn(color.r, color2.r, color.a, color2.a),
            g: rgb_fn(color.g, color2.g, color.a, color2.a),
            b: rgb_fn(color.b, color2.b, color.a, color2.a),
            a: a_fn(color.a, color2.a),
        }
    }

    /// Takes two [u8; 4]s, converts them to Rgba structs, then blends them according to blend_mode by calling blend().
    fn blend_u8(color: &[u8], other_color: &[u8], blend_mode: u8) -> [u8; 4] {
        Rgba::from_array(color)
            .blend(&Rgba::from_array(other_color), blend_mode)
            .into_array()
    }

    /// Blends two colors according to blend_mode. The numbers correspond to BYOND blend modes.
    fn blend(&self, other_color: &Rgba, blend_mode: u8) -> Rgba {
        match blend_mode {
            0 => Rgba::map_each(self, other_color, |c1, c2| c1 + c2, f32::min),
            1 => Rgba::map_each(self, other_color, |c1, c2| c2 - c1, f32::min),
            2 => Rgba::map_each(
                self,
                other_color,
                |c1, c2| c1 * c2 / 255.0,
                |a1: f32, a2: f32| a1 * a2 / 255.0,
            ),
            3 => Rgba::map_each_a(
                self,
                other_color,
                |c1, c2, _c1_a, c2_a| c1 + (c2 - c1) * c2_a / 255.0,
                |a1, a2| {
                    let high = f32::max(a1, a2);
                    let low = f32::min(a1, a2);
                    high + (high * low / 255.0)
                },
            ),
            6 => Rgba::map_each_a(
                other_color,
                self,
                |c1, c2, _c1_a, c2_a| c1 + (c2 - c1) * c2_a / 255.0,
                |a1, a2| {
                    let high = f32::max(a1, a2);
                    let low = f32::min(a1, a2);
                    high + (high * low / 255.0)
                },
            ),
            _ => self.clone(),
        }
    }
}
