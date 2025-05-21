use dmi::icon::Looping;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use tracy_full::zone;

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct UniversalIcon {
    pub icon_file: String,
    pub icon_state: String,
    pub dir: Option<u8>,
    pub frame: Option<u32>,
    pub transform: Vec<Transform>,
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
    /// Returns a new UniversalIcon that's a copy of the current one without any transforms
    pub fn to_base(&self) -> Self {
        UniversalIcon {
            icon_file: self.icon_file.to_owned(),
            icon_state: self.icon_state.to_owned(),
            dir: self.dir,
            frame: self.frame,
            transform: Vec::new(),
        }
    }

    /// Gives a list of nested icons within this UniversalIcon. Optionally returns a reference to the self at the start of the list.
    pub fn get_nested_icons(&self, include_self: bool) -> Vec<&UniversalIcon> {
        zone!("get_nested_icons");
        let mut icons: Vec<&UniversalIcon> = Vec::new();
        if include_self {
            icons.push(self);
        }
        for transform in &self.transform {
            if let Transform::BlendIcon { icon, .. } = transform {
                let nested = icon.get_nested_icons(true);
                for icon in nested {
                    icons.push(icon)
                }
            }
        }
        icons
    }
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(tag = "type")]
pub enum Transform {
    BlendColor { color: String, blend_mode: u8 },
    BlendIcon { icon: UniversalIcon, blend_mode: u8 },
    Scale { width: u32, height: u32 },
    Crop { x1: i32, y1: i32, x2: i32, y2: i32 },
}

#[derive(Clone)]
pub struct UniversalIconData {
    pub images: Vec<DynamicImage>,
    pub frames: u32,
    pub dirs: u8,
    pub delay: Option<Vec<f32>>,
    pub loop_flag: Looping,
    pub rewind: bool,
}
