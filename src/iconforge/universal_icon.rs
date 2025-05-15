use std::sync::Arc;

use crate::iconforge::{blending, icon_operations, image_cache};
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

impl Transform {
    /// Applies this transform to UniversalIconData. Optionally flattens to only the first dir and frame.
    pub fn apply(
        &self,
        image_data: Arc<UniversalIconData>,
        flatten: bool,
    ) -> Result<UniversalIconData, String> {
        zone!("transform_apply");
        let images: Vec<DynamicImage>;
        let mut frames = image_data.frames;
        let mut dirs = image_data.dirs;
        let mut delay = image_data.delay.to_owned();
        let loop_flag = image_data.loop_flag;
        let rewind = image_data.rewind;
        match &self {
            Transform::BlendColor { color, blend_mode } => {
                let blend_mode = &blending::BlendMode::from_u8(blend_mode)?;
                match icon_operations::blend_images_color(image_data.images.clone(), color, blend_mode)
                {
                    Ok(result_images) => {
                        images = result_images;
                    }
                    Err(err) => {
                        return Err(err.to_string());
                    }
                }
            }
            Transform::BlendIcon { icon, blend_mode } => {
                zone!("blend_icon");
                let (mut other_image_data, cached) = image_cache::universal_icon_to_images(
                    icon,
                    &format!("Transform blend_icon {icon}"),
                    true,
                    false,
                    flatten,
                )?;

                if !cached {
                    other_image_data =
                        apply_all_transforms(other_image_data, &icon.transform, flatten)?;
                };
                let new_out = icon_operations::blend_images_other_universal(
                    image_data,
                    other_image_data.clone(),
                    &blending::BlendMode::from_u8(blend_mode)?,
                )?;
                images = new_out.images;
                frames = new_out.frames;
                dirs = new_out.dirs;
                delay = new_out.delay;
                image_cache::cache_transformed_images(icon, other_image_data);
            }
            Transform::Scale { width, height } => {
                images = icon_operations::scale_images(image_data.images.clone(), *width, *height);
            }
            Transform::Crop { x1, y1, x2, y2 } => {
                images = icon_operations::crop_images(image_data.images.clone(), *x1, *y1, *x2, *y2)?;
            }
        }
        Ok(UniversalIconData {
            images,
            frames,
            dirs,
            delay,
            loop_flag,
            rewind,
        })
    }
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

/// Applies a list of Transforms to UniversalIconData immediately and sequentially, while handling any errors. Optionally flattens to only the first dir and frame.
fn apply_all_transforms(
    image_data: Arc<UniversalIconData>,
    transforms: &Vec<Transform>,
    flatten: bool,
) -> Result<Arc<UniversalIconData>, String> {
    let mut errors = Vec::<String>::new();
    let mut last_image_data = image_data;
    for transform in transforms {
        match transform.apply(last_image_data.clone(), flatten) {
            Ok(new_image_data) => last_image_data = Arc::new(new_image_data),
            Err(error) => errors.push(error),
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(last_image_data)
}
