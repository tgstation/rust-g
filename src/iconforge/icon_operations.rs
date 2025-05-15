use super::{
    blending, image_cache,
    universal_icon::{Transform, UniversalIconData},
};
use crate::error::Error;
use dmi::icon::IconState;
use image::{DynamicImage, Pixel, RgbaImage};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::{Arc, Mutex};
use tracy_full::zone;

pub fn blend_color(
    image: &mut RgbaImage,
    color: &String,
    blend_mode: &blending::BlendMode,
) -> Result<(), String> {
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
            return Err(format!("Decoding hex color {color} failed: {err}"));
        }
    }
    for x in 0..image.width() {
        for y in 0..image.height() {
            let px = image.get_pixel_mut(x, y);
            let pixel = px.channels();
            let blended = blending::Rgba::blend_u8(pixel, &color2, blend_mode);

            *px = image::Rgba::<u8>(blended);
        }
    }
    Ok(())
}

pub fn blend_icon(
    image: &mut RgbaImage,
    other_image: &RgbaImage,
    blend_mode: &blending::BlendMode,
) -> Result<(), String> {
    zone!("blend_icon");
    for x in 0..std::cmp::min(image.width(), other_image.width()) {
        for y in 0..std::cmp::min(image.height(), other_image.height()) {
            let px1 = image.get_pixel_mut(x, y);
            let px2 = other_image.get_pixel(x, y);
            let pixel_1 = px1.channels();
            let pixel_2 = px2.channels();

            let blended = blending::Rgba::blend_u8(pixel_1, pixel_2, blend_mode);

            *px1 = image::Rgba::<u8>(blended);
        }
    }
    Ok(())
}

pub fn crop(image: &mut RgbaImage, x1: i32, y1: i32, x2: i32, y2: i32) -> Result<(), String> {
    zone!("crop");

    let i_width = image.width();
    let i_height = image.height();

    if x2 <= (x1 - 1) || y2 <= (y1 - 1) {
        return Err(format!(
            "Invalid bounds {x1} {y1} to {x2} {y2} in crop transform"
        ));
    }

    let (mut x1, mut y1, mut x2, mut y2) =
        byond_crop_to_image_coords(i_height as i32, x1, y1, x2, y2);

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

        image::imageops::replace(&mut blank_img, image, x_offset as i64, y_offset as i64);
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
    Ok(())
}

pub fn byond_crop_to_image_coords(
    image_height: i32,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
) -> (i32, i32, i32, i32) {
    // BYOND indexes from 1,1! how silly of them. We'll just fix this here.
    // Crop(1,1,1,1) is a valid statement. Save us.
    // Convert from BYOND (0,0 is bottom left) to Rust (0,0 is top left)
    // BYOND also includes the upper bound
    (x1 - 1, image_height - y2, x2, image_height - (y1 - 1))
}

pub fn scale(image: &mut RgbaImage, width: u32, height: u32) {
    zone!("scale");
    let old_width = image.width() as usize;
    let old_height = image.height() as usize;
    let x_ratio = old_width as f32 / width as f32;
    let y_ratio = old_height as f32 / height as f32;
    let mut new_image = RgbaImage::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let old_x = (x as f32 * x_ratio).floor() as u32;
            let old_y = (y as f32 * y_ratio).floor() as u32;
            new_image.put_pixel(x, y, *image.get_pixel(old_x, old_y));
        }
    }
    *image = new_image;
}

/// Scales a set of images.
pub fn scale_images(images: Vec<DynamicImage>, width: u32, height: u32) -> Vec<DynamicImage> {
    zone!("scale_images");
    images
        .into_par_iter()
        .map(|image: DynamicImage| {
            zone!("scale_image");
            let mut new_image = image.clone().into_rgba8();
            scale(&mut new_image, width, height);
            DynamicImage::ImageRgba8(new_image)
        })
        .collect()
}

/// Crops a set of images.
pub fn crop_images(
    images: Vec<DynamicImage>,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
) -> Result<Vec<DynamicImage>, Error> {
    zone!("crop_images");
    let errors = Arc::new(Mutex::new(Vec::<String>::new()));
    let images_out = images
        .into_par_iter()
        .map(|image| {
            zone!("crop_image");
            let mut new_image = image.clone().into_rgba8();
            if let Err(err) = crop(&mut new_image, x1, y1, x2, y2) {
                errors.lock().unwrap().push(err);
            }
            DynamicImage::ImageRgba8(new_image)
        })
        .collect();
    let errors_unlock = errors.lock().unwrap();
    if !errors_unlock.is_empty() {
        return Err(Error::IconForge(errors_unlock.join("\n")));
    }
    Ok(images_out)
}

/// Blends a set of images with a color.
pub fn blend_images_color(
    images: Vec<DynamicImage>,
    color: &String,
    blend_mode: &blending::BlendMode,
) -> Result<Vec<DynamicImage>, Error> {
    zone!("blend_images_color");
    let errors = Arc::new(Mutex::new(Vec::<String>::new()));
    let images_out = images
        .into_par_iter()
        .map(|image| {
            zone!("blend_image_color");
            let mut new_image = image.clone().into_rgba8();
            if let Err(err) = blend_color(&mut new_image, color, blend_mode) {
                errors.lock().unwrap().push(err);
            }
            DynamicImage::ImageRgba8(new_image)
        })
        .collect();
    let errors_unlock = errors.lock().unwrap();
    if !errors_unlock.is_empty() {
        return Err(Error::IconForge(errors_unlock.join("\n")));
    }
    Ok(images_out)
}

/// Blends a set of images with another set of images.
/// The frame and dir counts of first_matched_state are mutated to match the new icon.
pub fn blend_images_other_universal(
    image_data: Arc<UniversalIconData>,
    image_data_other: Arc<UniversalIconData>,
    blend_mode: &blending::BlendMode,
) -> Result<UniversalIconData, Error> {
    zone!("blend_images_other_universal");
    let errors = Arc::new(Mutex::new(Vec::<String>::new()));
    let expected_length_first = image_data.dirs as u32 * image_data.frames;
    // Make sure our logic sound... First and last should correctly match these two Vecs at all times, but this assumption might be incorrect.
    if expected_length_first != image_data.images.len() as u32 {
        return Err(Error::IconForge(format!(
            "Error during blend_images_other - the base set of images did not contain the correct amount of images (contains {}, it should contain {expected_length_first}) to match the amount of dirs ({}) or frames ({}) from the first icon state. This shouldn't ever happen!",
            image_data.images.len(), image_data.dirs, image_data.frames
        )));
    }
    let expected_length_last = image_data_other.dirs as u32 * image_data_other.frames;
    if expected_length_last != image_data_other.images.len() as u32 {
        return Err(Error::IconForge(format!(
            "Error during blend_images_other - the blending set of images did not contain the correct amount of images (contains {}, it should contain {expected_length_last}) to match the amount of dirs ({}) or frames ({}) from the last icon state. This shouldn't ever happen!",
            image_data_other.images.len(), image_data_other.dirs, image_data_other.frames
        )));
    }
    let mut images = image_data.images.clone();
    let mut images_other = image_data_other.images.clone();
    let mut frames_out = image_data.frames;
    let mut delay_out = image_data.delay.clone();
    // If more custom handling is added in the future, this could be mutable
    let /*mut*/ dirs_out = image_data.dirs;

    // Now we can complain to the user to handle a difference in length.
    if image_data.dirs != image_data_other.dirs {
        // We can handle the specific case where there's only one dir being blended onto multiple. Copy the icon for each frame onto all dirs.
        if image_data.dirs > image_data_other.dirs && image_data_other.dirs == 1 {
            // Loop backwards so that the frame indexes remain consistent while we iterate, since inserts shift the array right
            for i in (0..(image_data_other.frames)).rev() {
                // Add the missing dirs between frames
                for _ in 0..(image_data.dirs - 1) {
                    // Insert after the current frame index
                    images_other.insert(
                        (i + 1) as usize,
                        images_other.get(i as usize).unwrap().clone(),
                    );
                }
            }
        } else {
            return Err(Error::IconForge(format!(
                "Attempted to blend two icon states with different dir amounts with {} and {} dirs respectively.",
                image_data.dirs, image_data_other.dirs
            )));
        }
    }

    if image_data.frames != image_data_other.frames {
        // We can handle the specific case where there's only one frame on the base and the other has more frames. Simply add copies of that first frame.
        if image_data_other.frames > 1 && image_data.frames == 1 {
            for _ in 0..(image_data_other.frames - 1) {
                // Copy all dirs for each frame
                for i in 0..(image_data.dirs) {
                    images.push(images.get(i as usize).unwrap().clone());
                }
            }
            // Update the output IconState's frame count, because the values from the first state are used for the final result.
            frames_out = image_data_other.frames;
            // Copy the delays as well
            delay_out = image_data_other.delay.to_owned();
        } else {
            return Err(Error::IconForge(format!(
                "Attempted to blend two icon states with different frame amounts - with {} and {} frames respectively.",
                image_data.frames, image_data_other.frames
            )));
        }
    }
    let images_out: Vec<DynamicImage> = if images_other.len() == 1 {
        // This is useful in the case where the something with 4+ dirs blends with 1dir
        let first_image = images_other.first().unwrap().clone().into_rgba8();
        images
            .into_par_iter()
            .map(|image| {
                zone!("blend_image_other_simple");
                let mut new_image = image.clone().into_rgba8();
                match blend_icon(&mut new_image, &first_image, blend_mode) {
                    Ok(_) => (),
                    Err(error) => {
                        errors.lock().unwrap().push(error);
                    }
                };
                DynamicImage::ImageRgba8(new_image)
            })
            .collect()
    } else {
        (images, images_other)
            .into_par_iter()
            .map(|(image, image2)| {
                zone!("blend_image_other");
                let mut new_image = image.clone().into_rgba8();
                match blend_icon(&mut new_image, &image2.into_rgba8(), blend_mode) {
                    Ok(_) => (),
                    Err(error) => {
                        errors.lock().unwrap().push(error);
                    }
                };
                DynamicImage::ImageRgba8(new_image)
            })
            .collect()
    };
    let errors_unlock = errors.lock().unwrap();
    if !errors_unlock.is_empty() {
        return Err(Error::IconForge(errors_unlock.join("\n")));
    }
    Ok(UniversalIconData {
        images: images_out,
        frames: frames_out,
        dirs: dirs_out,
        delay: delay_out,
        loop_flag: image_data.loop_flag,
        rewind: image_data.rewind,
    })
}

/// Blends a set of images with another set of images.
/// The frame and dir counts of first_matched_state are mutated to match the new icon.
pub fn blend_images_other(
    images: Vec<DynamicImage>,
    images_other: Vec<DynamicImage>,
    blend_mode: &blending::BlendMode,
    first_matched_state: &mut Option<IconState>,
    last_matched_state: &mut Option<IconState>,
) -> Result<Vec<DynamicImage>, Error> {
    zone!("blend_images_other");
    let base_icon_state = match first_matched_state {
        Some(state) => state,
        None => {
            return Err(Error::IconForge("No value in first_matched_state during blend_images_other. This should never happen, unless a GAGS config doesn't start with an icon_state.".to_string()));
        }
    };
    let blending_icon_state = match last_matched_state {
        Some(state) => state,
        None => {
            return Err(Error::IconForge("No value in last_matched_state during blend_images_other. This should never happen, unless a GAGS config doesn't start with an icon_state.".to_string()));
        }
    };
    let errors = Arc::new(Mutex::new(Vec::<String>::new()));
    let expected_length_first = base_icon_state.dirs as u32 * base_icon_state.frames;
    // Make sure our logic sound... First and last should correctly match these two Vecs at all times, but this assumption might be incorrect.
    if expected_length_first != images.len() as u32 {
        return Err(Error::IconForge(format!(
            "Error during blend_images_other - the base set of images did not contain the correct amount of images (contains {}, it should contain {expected_length_first}) to match the amount of dirs ({}) or frames ({}) from the first icon state. This shouldn't ever happen!",
            images.len(), base_icon_state.dirs, base_icon_state.frames
        )));
    }
    let expected_length_last = blending_icon_state.dirs as u32 * blending_icon_state.frames;
    if expected_length_last != images_other.len() as u32 {
        return Err(Error::IconForge(format!(
            "Error during blend_images_other - the blending set of images did not contain the correct amount of images (contains {}, it should contain {expected_length_last}) to match the amount of dirs ({}) or frames ({}) from the last icon state. This shouldn't ever happen!",
            images_other.len(), blending_icon_state.dirs, blending_icon_state.frames
        )));
    }
    let mut images = images.clone();
    let mut images_other = images_other.clone();
    // Now we can complain to the user to handle a difference in length.
    if base_icon_state.dirs != blending_icon_state.dirs {
        // We can handle the specific case where there's only one dir being blended onto multiple. Copy the icon for each frame onto all dirs.
        if base_icon_state.dirs > blending_icon_state.dirs && blending_icon_state.dirs == 1 {
            // Loop backwards so that the frame indexes remain consistent while we iterate, since inserts shift the array right
            for i in (0..(blending_icon_state.frames)).rev() {
                // Add the missing dirs between frames
                for _ in 0..(base_icon_state.dirs - 1) {
                    // Insert after the current frame index
                    images_other.insert(
                        (i + 1) as usize,
                        images_other.get(i as usize).unwrap().clone(),
                    );
                }
            }
            // Copy the dir amount in case we need to handle frame cases next.
            blending_icon_state.dirs = base_icon_state.dirs;
        } else {
            return Err(Error::IconForge(format!(
                "Attempted to blend two icon states with different dir amounts - {} and {}, with {} and {} dirs respectively.",
                base_icon_state.name, blending_icon_state.name, base_icon_state.dirs, blending_icon_state.dirs
            )));
        }
    }

    if base_icon_state.frames != blending_icon_state.frames {
        // We can handle the specific case where there's only one frame on the base and the other has more frames. Simply add copies of that first frame.
        if blending_icon_state.frames > 1 && base_icon_state.frames == 1 {
            for _ in 0..(blending_icon_state.frames - 1) {
                // Copy all dirs for each frame
                for i in 0..(base_icon_state.dirs) {
                    images.push(images.get(i as usize).unwrap().clone());
                }
            }
            // Update the output IconState's frame count, because the values from the first state are used for the final result.
            base_icon_state.frames = blending_icon_state.frames;
            // Copy the delays as well
            base_icon_state.delay = blending_icon_state.delay.to_owned();
        } else {
            return Err(Error::IconForge(format!(
                "Attempted to blend two icon states with different frame amounts - {} and {}, with {} and {} frames respectively.",
                base_icon_state.name, blending_icon_state.name, base_icon_state.frames, blending_icon_state.frames
            )));
        }
    }
    let images_out: Vec<DynamicImage> = if images_other.len() == 1 {
        // This is useful in the case where the something with 4+ dirs blends with 1dir
        let first_image = images_other.first().unwrap().clone().into_rgba8();
        images
            .into_par_iter()
            .map(|image| {
                zone!("blend_image_other_simple");
                let mut new_image = image.clone().into_rgba8();
                match blend_icon(&mut new_image, &first_image, blend_mode) {
                    Ok(_) => (),
                    Err(error) => {
                        errors.lock().unwrap().push(error);
                    }
                };
                DynamicImage::ImageRgba8(new_image)
            })
            .collect()
    } else {
        (images, images_other)
            .into_par_iter()
            .map(|(image, image2)| {
                zone!("blend_image_other");
                let mut new_image = image.clone().into_rgba8();
                match blend_icon(&mut new_image, &image2.into_rgba8(), blend_mode) {
                    Ok(_) => (),
                    Err(error) => {
                        errors.lock().unwrap().push(error);
                    }
                };
                DynamicImage::ImageRgba8(new_image)
            })
            .collect()
    };
    let errors_unlock = errors.lock().unwrap();
    if !errors_unlock.is_empty() {
        return Err(Error::IconForge(errors_unlock.join("\n")));
    }
    Ok(images_out)
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
                match blend_images_color(image_data.images.clone(), color, blend_mode) {
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
                let (mut other_image_data, cached) = icon.get_image_data(
                    &format!("Transform blend_icon {icon}"),
                    true,
                    false,
                    flatten,
                )?;

                if !cached {
                    other_image_data =
                        apply_all_transforms(other_image_data, &icon.transform, flatten)?;
                };
                let new_out = blend_images_other_universal(
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
                images = scale_images(image_data.images.clone(), *width, *height);
            }
            Transform::Crop { x1, y1, x2, y2 } => {
                images = crop_images(image_data.images.clone(), *x1, *y1, *x2, *y2)?;
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
