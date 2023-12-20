// DMI spritesheet generator
// Developed by itsmeow
use crate::jobs;
use crate::error::Error;
use std::{
    fs::File,
    io::BufReader,
};
use dmi::icon::{Icon, IconState};
use image::{DynamicImage, GenericImage, GenericImageView};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
//use raster::Image;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracy_full::{zone, frame};
use once_cell::sync::OnceCell;

fn icon_file_to_icon() -> &'static Mutex<HashMap<String, Icon>> {
    static INSTANCE: OnceCell<Mutex<HashMap<String, Icon>>> = OnceCell::new();
    INSTANCE.get_or_init(|| Mutex::new(HashMap::new()))
}

byond_fn!(fn iconforge_generate(file_path, spritesheet_name, sprites) {
    catch_panic(file_path, spritesheet_name, sprites).err()
});


byond_fn!(fn iconforge_generate_async(file_path, spritesheet_name, sprites) {
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
        x1: u32,
        y1: u32,
        x2: u32,
        y2: u32,
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

    let size_to_images: Arc<Mutex<HashMap<String, Vec<DynamicImage>>>> = Arc::new(Mutex::new(HashMap::new()));
    let sprites_map: HashMap<String, IconObject> = serde_json::from_str::<HashMap<String, IconObject>>(sprites)?;
    let sprites_objects: Arc<Mutex<HashMap<String, SpritesheetEntry>>> = Arc::new(Mutex::new(HashMap::new()));

    sprites_map.par_iter().for_each(|sprite_entry| {
        zone!("sprite_to_icon");
        let (_, icon) = sprite_entry;
        let icon_path = icon.icon_file.to_owned();
        if icon_file_to_icon().lock().unwrap().contains_key(&icon_path) {
            return;
        }
        let reader = BufReader::new(File::open(&icon_path).unwrap());
        let icon: Option<Icon>;
        {
            zone!("load_icon");
            icon = Icon::load(reader).ok();
        }
        if icon.is_none() {
            error.lock().unwrap().push(format!("Invalid DMI: {}", icon_path));
            return;
        }
        icon_file_to_icon().lock().unwrap().insert(icon_path, icon.unwrap().to_owned());
    });

    sprites_map.par_iter().for_each(|sprite_entry| {
        zone!("map_sprite");
        let (sprite_name, icon) = sprite_entry;
        let parsed_icon = icon_file_to_icon().lock().unwrap().get(&icon.icon_file).unwrap().to_owned();
        let mut matched_state: Option<IconState> = Option::None;
        for icon_state in parsed_icon.states {
            if icon_state.name == icon.icon_state {
                matched_state = Option::Some(icon_state.clone());
                break;
            }
        }
        if matched_state.is_none() {
            error.lock().unwrap().push(format!("Could not find associated icon state {} for {}", icon.icon_state, sprite_name));
            return;
        }
        let state = matched_state.unwrap();
        if !( if icon.dir == 2 { state.dirs >= 1 } else { state.dirs >= 4 } && state.frames >= icon.frame ) {
            error.lock().unwrap().push(format!("Could not find associated dir or frame dir: {} frame: {} in {} icon_state - dirs: {} frames: {}", icon.dir, icon.frame, sprite_name, state.dirs, state.frames));
            return;
        }
        let mut icon_idx: u32 = 0;
        if state.dirs == 4 {
            icon_idx = match icon.dir {
                 2 => 0, // South
                1 => 1, // North
                4 => 2, // East
                8 => 3, // West
                _ => 0,
            }
        } else if state.dirs != 1 {
            error.lock().unwrap().push(format!("Unsupported dirs size of {} in {} state: {} for sprite {}", state.dirs, icon.icon_file, icon.icon_state, sprite_name));
            return;
        }
        if state.frames > 1 {
            // Add one so zero scales properly
            icon_idx = (icon_idx + 1) * icon.frame - 1
        }
        let image: &DynamicImage = state.images.get(usize::try_from(icon_idx).unwrap()).unwrap();
        let cloned_image: DynamicImage = image.clone();
        // apply transforms here
        let size_id = format!("{}x{}", cloned_image.width(), cloned_image.height());
        let mut size_map = size_to_images.lock().unwrap();
        let vec = (*size_map).entry(size_id.to_owned()).or_insert(Vec::new());
        vec.push(cloned_image);
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

    let sizes: Vec<String> = size_to_images.lock().unwrap().clone().into_keys().collect();

    let returned = Returned {
        sizes: sizes,
        sprites: sprites_objects.lock().unwrap().to_owned(),
        error: error.lock().unwrap().join("\n"),
    };
    Ok(serde_json::to_string::<Returned>(&returned).unwrap())
}
