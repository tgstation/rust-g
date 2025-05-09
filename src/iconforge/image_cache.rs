use crate::error::Error;
use crate::iconforge::UniversalIcon;
use dashmap::DashMap;
use dmi::icon::Icon;
use dmi::{dirs::Dirs, icon::IconState};
use image::RgbaImage;
use once_cell::sync::Lazy;
use std::fs::File;
use std::hash::BuildHasherDefault;
use std::io::BufReader;
use std::sync::Arc;
use tracy_full::zone;
use twox_hash::XxHash64;

/// A cache of UniversalIcon to RgbaImage (with transforms applied! This can only contain COMPLETED sprites).
static ICON_STATES: Lazy<DashMap<UniversalIcon, RgbaImage, BuildHasherDefault<XxHash64>>> =
    Lazy::new(|| DashMap::with_hasher(BuildHasherDefault::<XxHash64>::default()));

pub fn image_cache_contains(icon: &UniversalIcon) -> bool {
    ICON_STATES.contains_key(icon)
}

pub fn image_cache_clear() {
    ICON_STATES.clear();
}

/// Takes an UniversalIcon, gets its DMI, then picks out a RgbaImage for the IconState.
/// Returns with True if the RgbaImage is pre-cached (and shouldn't have new transforms applied)
/// Gives ownership over the image. Please return when you are done <3 (via cache::return_image)
pub fn icon_to_image(
    icon: &UniversalIcon,
    sprite_name: &String,
    cached: bool,
    must_be_cached: bool,
) -> Result<(RgbaImage, bool), String> {
    zone!("icon_to_image");
    if cached {
        zone!("check_rgba_image_exists");
        if let Some(entry) = ICON_STATES.get(icon) {
            return Ok((entry.value().clone(), true));
        }
        if must_be_cached {
            return Err(String::from("Image not found in cache!"));
        }
    }
    let dmi = filepath_to_dmi(&icon.icon_file)?;
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
                "Could not find associated icon state {} for {sprite_name}",
                icon.icon_state
            ));
        }
    };

    let dir = match Dirs::from_bits(icon.dir.unwrap_or(1)) {
        Some(dir) => dir,
        None => {
            return Err(format!("Invalid dir number {} for {sprite_name}", icon.dir.unwrap_or(1)));
        }
    };
    Ok(match state.get_image(&dir, icon.frame.unwrap_or(1)) {
        Ok(image) => (image.to_rgba8(), false),
        Err(err) => {
            return Err(format!("Error getting image for {sprite_name}: {err}"));
        }
    })
}

/// Gives an image back to the cache, after it is done being used.
pub fn return_image(image: RgbaImage, icon: &UniversalIcon) -> Result<(), Error> {
    zone!("insert_rgba_image");
    ICON_STATES.insert(icon.to_owned(), image);
    Ok(())
}

/* ---- DMI CACHING ---- */

/// A cache of DMI filepath -> Icon objects.
static ICON_FILES: Lazy<DashMap<String, Arc<Icon>, BuildHasherDefault<XxHash64>>> =
    Lazy::new(|| DashMap::with_hasher(BuildHasherDefault::<XxHash64>::default()));

pub fn icon_cache_clear() {
    ICON_FILES.clear();
}

/// Given a DMI filepath, returns a DMI Icon structure and caches it.
pub fn filepath_to_dmi(icon_path: &str) -> Result<Arc<Icon>, String> {
    zone!("filepath_to_dmi");
    {
        zone!("check_dmi_exists");
        if let Some(found) = ICON_FILES.get(icon_path) {
            return Ok(found.clone());
        }
    }
    let icon_file = match File::open(icon_path) {
        Ok(icon_file) => icon_file,
        Err(err) => {
            return Err(format!("Failed to open DMI '{icon_path}' - {err}"));
        }
    };
    let reader = BufReader::new(icon_file);
    let dmi: Icon;
    {
        zone!("parse_dmi");
        dmi = match Icon::load(reader) {
            Ok(dmi) => dmi,
            Err(err) => {
                return Err(format!("DMI '{icon_path}' failed to parse - {err}"));
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
