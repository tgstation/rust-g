use super::universal_icon::{UniversalIcon, UniversalIconData};
use dashmap::DashMap;
use dmi::{
    dirs::{ALL_DIRS, CARDINAL_DIRS, Dirs},
    icon::{Icon, IconState, dir_to_dmi_index},
};
use image::RgbaImage;
use once_cell::sync::Lazy;
use std::{fs::File, hash::BuildHasherDefault, io::BufReader, sync::Arc};
use tracy_full::zone;
use twox_hash::XxHash64;

/// A cache of UniversalIcon to UniversalIconData. In order for something to exist in this cache, it must have had any transforms applied to the images.
static ICON_STATES: Lazy<
    DashMap<UniversalIcon, Arc<UniversalIconData>, BuildHasherDefault<XxHash64>>,
> = Lazy::new(|| DashMap::with_hasher(BuildHasherDefault::<XxHash64>::default()));

static ICON_STATES_FLAT: Lazy<
    DashMap<UniversalIcon, Arc<UniversalIconData>, BuildHasherDefault<XxHash64>>,
> = Lazy::new(|| DashMap::with_hasher(BuildHasherDefault::<XxHash64>::default()));

pub fn image_cache_contains(icon: &UniversalIcon, flatten: bool) -> bool {
    if flatten {
        ICON_STATES_FLAT.contains_key(icon)
    } else {
        ICON_STATES.contains_key(icon)
    }
}

pub fn image_cache_clear() {
    ICON_STATES.clear();
    ICON_STATES_FLAT.clear();
}

impl UniversalIcon {
    /// Gets this icon's associated DMI, then picks out a UniversalIconData for the IconState.
    /// If flatten is true, will output only one dir and frame (defaulting to SOUTH/1 if unscoped) regardless of the input uni_icon
    /// Returns with True if the UniversalIconData is pre-cached (and shouldn't have new transforms applied)
    pub fn get_image_data(
        &self,
        sprite_name: &String,
        cached: bool,
        must_be_cached: bool,
        flatten: bool,
    ) -> Result<(Arc<UniversalIconData>, bool), String> {
        zone!("universal_icon_to_image_data");
        if cached {
            zone!("check_image_cache");
            if let Some(entry) = if flatten {
                ICON_STATES_FLAT.get(self)
            } else {
                ICON_STATES.get(self)
            } {
                return Ok((entry.value().to_owned(), true));
            }
            if must_be_cached {
                return Err(format!(
                    "Image was requested but does not exist in the cache. It's likely that the icon state doesn't exist: {self} - while generating '{sprite_name}'"
                ));
            }
        }
        let dmi = filepath_to_dmi(&self.icon_file)?;
        let mut matched_state: Option<&IconState> = None;
        {
            zone!("match_icon_state");
            for icon_state in &dmi.states {
                if icon_state.name == self.icon_state {
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
                    self.icon_state
                ));
            }
        };

        let mut dirs = state.dirs as usize;
        let mut dir_index = 0;

        if let Some(dir_bits) = self.dir {
            // Consider 0 to be "unscoped"
            if dir_bits > 0 {
                dirs = 1;
                dir_index = match Dirs::from_bits(dir_bits) {
                    Some(dir) => {
                        if (state.dirs == 1 && dir != Dirs::SOUTH)
                            || (state.dirs == 4 && !CARDINAL_DIRS.contains(&dir))
                            || (state.dirs == 8 && !ALL_DIRS.contains(&dir))
                        {
                            return Err(format!(
                                "Dir specified {dir} is not in the set of valid dirs ({} dirs) for icon_state \"{}\" for {sprite_name}",
                                state.dirs, state.name
                            ));
                        }
                        match dir_to_dmi_index(&dir) {
                            Some(index) => index,
                            None => {
                                return Err(format!(
                                    "Invalid dir in dir ordering {dir} for {sprite_name}"
                                ));
                            }
                        }
                    }
                    None => {
                        return Err(format!("Invalid dir number {dir_bits} for {sprite_name}"));
                    }
                };
            } else if flatten {
                dirs = 1;
                dir_index = 0;
            }
        } else if flatten {
            dirs = 1;
            dir_index = 0;
        }

        let mut frames = state.frames as usize;
        let mut frame_offset: usize = 0;

        if let Some(frame) = self.frame {
            // Consider 0 to be "unscoped"
            // Also no underflow please
            if frame > 0 {
                frames = 1;
                frame_offset = frame as usize - 1;
                if state.frames < frame {
                    return Err(format!(
                        "Specified frame \"{frame}\" is larger than the number of frames ({}) for icon_state \"{}\" in sprite \"{sprite_name}\"",
                        state.frames, state.name
                    ));
                }
            } else if flatten {
                frames = 1;
                frame_offset = 0;
            }
        } else if flatten {
            frames = 1;
            frame_offset = 0;
        }

        let mut images: Vec<RgbaImage> = Vec::new();

        for frame_index in frame_offset..(frame_offset + frames) {
            for dir_offset in dir_index..(dir_index + dirs) {
                match state
                    .images
                    .get((frame_index * state.dirs as usize) + dir_offset)
                {
                    Some(image) => images.push(image.clone()),
                    None => {
                        return Err(format!(
                            "Somehow got out of bounds image for dir {dir_index} and frame {frame_offset} on {sprite_name}!"
                        ));
                    }
                }
            }
        }

        let result = Arc::new(UniversalIconData {
            images,
            frames: frames as u32,
            dirs: dirs as u8,
            delay: if frames > 1 {
                state.delay.to_owned()
            } else {
                None
            },
            loop_flag: state.loop_flag,
            rewind: state.rewind,
        });

        // Don't insert into the cache here, because the cache should only contain transformed images.
        Ok((result, false))
    }
}

pub fn cache_transformed_images(
    uni_icon: &UniversalIcon,
    image_data: Arc<UniversalIconData>,
    flatten: bool,
) {
    zone!("cache_transformed_images");
    if flatten {
        ICON_STATES_FLAT.insert(uni_icon.to_owned(), image_data.to_owned());
    } else {
        ICON_STATES.insert(uni_icon.to_owned(), image_data.to_owned());
    }
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
