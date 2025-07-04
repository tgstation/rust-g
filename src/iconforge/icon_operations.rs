use super::{
    blending, image_cache,
    universal_icon::{Transform, UniversalIconData},
};
use crate::error::Error;
use dmi::{
    dirs::Dirs,
    icon::IconState,
};
use image::{imageops, DynamicImage, Rgba, RgbaImage};
use ordered_float::OrderedFloat;
use rayon::{
    iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
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
    let image_buf: &mut [u8] = image.as_mut();
    image_buf.par_chunks_exact_mut(4).for_each(|px| {
        let blended = blend_mode.blend_u8(px, &color2);
        px.copy_from_slice(&blended);
    });
    Ok(())
}

pub fn blend_icon(
    image: &mut RgbaImage,
    other_image: &RgbaImage,
    blend_mode: &blending::BlendMode,
    position: Option<(i32, i32)>,
) -> Result<(), String> {
    let (x_byond_offset, y_byond_offset) = position.unwrap_or((1, 1));
    let (x_offset, y_offset) = convert_byond_image_offset(x_byond_offset, y_byond_offset);

    let image_width = image.width() as i32;
    let image_height = image.height() as i32;
    let other_width = other_image.width() as i32;
    let other_height = other_image.height() as i32;

    // Convert from bottom-left Y to top-left Y
    let y_offset_adjusted = image_height.saturating_sub(other_height + y_offset);

    let image_buf: &mut [u8] = image.as_mut();
    let other_buf = other_image.as_flat_samples().samples;

    for y in 0..other_height {
        for x in 0..other_width {
            let target_x = x + x_offset;
            let target_y = y + y_offset_adjusted;

            // Skip all out-of-bounds blending
            if target_x < 0 || target_x >= image_width || target_y < 0 || target_y >= image_height {
                continue;
            }

            let target_index = ((target_y * image_width + target_x) * 4) as usize;
            let source_index = ((y * other_width + x) * 4) as usize;

            let px1 = &mut image_buf[target_index..target_index + 4];
            let px2 = &other_buf[source_index..source_index + 4];

            let blended = blend_mode.blend_u8(px1, px2);
            px1.copy_from_slice(&blended);
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
        convert_byond_crop_image_coords(i_height as i32, x1, y1, x2, y2);

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

pub fn convert_byond_image_offset(x_offset: i32, y_offset: i32) -> (i32, i32) {
    (x_offset - 1, y_offset - 1)
}

pub fn convert_byond_crop_image_coords(
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

pub fn scale(image: &mut RgbaImage, target_width: u32, target_height: u32) {
    zone!("scale");
    let original_width = image.width();
    let original_height = image.height();
    if target_width == original_width && target_height == original_height {
        return;
    }
    let upscale_x = target_width >= original_width;
    let upscale_y = target_height >= original_height;

    let mut output = RgbaImage::new(target_width, target_height);

    for ty in 0..target_height {
        for tx in 0..target_width {
            let (x0, x1) = if upscale_x {
                let sx = (tx as f32 * original_width as f32 / target_width as f32).floor() as u32;
                (sx, sx + 1)
            } else {
                let sx0 = (tx as f32 * original_width as f32 / target_width as f32).floor() as u32;
                let sx1 = ((((tx + 1) as f32 * original_width as f32) / target_width as f32).ceil()
                    as u32)
                    .min(original_width);
                (sx0, sx1)
            };

            let (y0, y1) = if upscale_y {
                let sy = (ty as f32 * original_height as f32 / target_height as f32).floor() as u32;
                (sy, sy + 1)
            } else {
                let sy0 =
                    (ty as f32 * original_height as f32 / target_height as f32).floor() as u32;
                let sy1 = ((((ty + 1) as f32 * original_height as f32) / target_height as f32)
                    .ceil() as u32)
                    .min(original_height);
                (sy0, sy1)
            };

            let mut acc_r = 0u32;
            let mut acc_g = 0u32;
            let mut acc_b = 0u32;
            let mut acc_a = 0u32;
            let mut contributing = 0u32;

            for y in y0..y1.min(original_height) {
                for x in x0..x1.min(original_width) {
                    let [r, g, b, a] = image.get_pixel(x, y).0;
                    if a > 0 {
                        acc_r += r as u32;
                        acc_g += g as u32;
                        acc_b += b as u32;
                        acc_a += a as u32;
                        contributing += 1;
                    }
                }
            }

            let area = (x1 - x0).max(1) * (y1 - y0).max(1);

            let pixel = if contributing > 0 {
                Rgba([
                    (acc_r / contributing) as u8,
                    (acc_g / contributing) as u8,
                    (acc_b / contributing) as u8,
                    (acc_a / area) as u8,
                ])
            } else {
                Rgba([0, 0, 0, 0])
            };

            output.put_pixel(tx, ty, pixel);
        }
    }

    *image = output
}

pub fn flip(image: &mut RgbaImage, dir: Dirs) {
    match dir {
        Dirs::NORTHEAST | Dirs::SOUTHWEST => {
            *image = imageops::rotate90(image);
            *image = imageops::flip_horizontal(image);
        }
        Dirs::NORTHWEST | Dirs::SOUTHEAST => {
            *image = imageops::rotate90(image);
            *image = imageops::flip_vertical(image);
        }
        Dirs::NORTH | Dirs::SOUTH => {
            *image = imageops::flip_vertical(image);
        }
        Dirs::EAST | Dirs::WEST => {
            *image = imageops::flip_horizontal(image);
        }
        _ => {}
    }
}

pub fn turn(image: &mut RgbaImage, angle: f32) {
    // Optimized rotations
    match angle {
        -360.0 | 360.0 | 0.0 => {
            return;
        }
        90.0 | -270.0 => {
            *image = imageops::rotate90(image);
            return;
        },
        270.0 | -90.0 => {
            *image = imageops::rotate270(image);
            return;
        },
        -180.0 | 180.0 => {
            *image = imageops::rotate180(image);
            return;
        }
        _ => {}
    }
    // Arbitrary rotations
    let rad = -angle.to_radians();
    let sin_rad = rad.sin();
    let cos_rad = rad.cos();
    let image_width = image.width();
    let image_height = image.height();
    let center_x = (image_width - 1) as f32 / 2.0;
    let center_y = (image_height - 1) as f32 / 2.0;
    let src_buf = image.as_raw();
    let mut output = RgbaImage::new(image_width, image_height);
    let output_buf: &mut [u8] = output.as_mut();

    output_buf
        .par_chunks_exact_mut(4)
        .enumerate()
        .for_each(|(i, out_pixel)| {
            let dst_x = (i % image_width as usize) as f32;
            let dst_y = (i / image_width as usize) as f32;
            let dx = dst_x - center_x;
            let dy = dst_y - center_y;
            let src_x = (dx * cos_rad - dy * sin_rad + center_x).round() as i32;
            let src_y = (dx * sin_rad + dy * cos_rad + center_y).round() as i32;
            if src_x >= 0 && src_x < image_width as i32 && src_y >= 0 && src_y < image_height as i32 {
                let src_i = (src_y as usize * image_width as usize + src_x as usize) * 4;
                out_pixel.copy_from_slice(&src_buf[src_i..src_i + 4]);
            } else {
                out_pixel.copy_from_slice(&[0, 0, 0, 0]);
            }
        });
    *image = output
}

pub fn shift(image: &mut RgbaImage, dir: Dirs, offset: i32, wrap: bool) {
    if offset == 0 {
        return;
    }
    let image_width = image.width();
    let image_height = image.height();
    let off_x = match dir {
        Dirs::EAST | Dirs::NORTHEAST | Dirs::SOUTHEAST => -1,
        Dirs::WEST | Dirs::NORTHWEST | Dirs::SOUTHWEST => 1,
        _ => 0,
    } * offset;
    let off_y = match dir {
        Dirs::NORTH | Dirs::NORTHEAST | Dirs::NORTHWEST => 1,
        Dirs::SOUTH | Dirs::SOUTHEAST | Dirs::SOUTHWEST => -1,
        _ => 0,
    } * offset;
    let mut output = RgbaImage::new(image_width, image_height);
    let output_buf: &mut [u8] = output.as_mut();
    let src_buf = image.as_raw();
    output_buf.par_chunks_exact_mut(4).enumerate().for_each(|(i, px)| {
        let dst_x = i as i32 % image_width as i32;
        let dst_y = i as i32 / image_width as i32;
        let mut src_x = dst_x + off_x;
        let mut src_y = dst_y + off_y;
        if src_x < 0 || src_x >= image_width as i32 || src_y < 0 || src_y >= image_height as i32 {
            if wrap {
                src_x = src_x.rem_euclid(image_width as i32);
                src_y = src_y.rem_euclid(image_height as i32);
            } else {
                px.copy_from_slice(&[0, 0, 0, 0]);
                return;
            }
        }
        let src_i = (src_y as usize * image_width as usize + src_x as usize) * 4;
        px.copy_from_slice(&src_buf[src_i..src_i+4]);
    });
    *image = output
}

#[rustfmt::skip]
pub fn map_colors(
    image: &mut RgbaImage,
    rr: f32, rg: f32, rb: f32, ra: Option<f32>,
    gr: f32, gg: f32, gb: f32, ga: Option<f32>,
    br: f32, bg: f32, bb: f32, ba: Option<f32>,
    ar: Option<f32>, ag: Option<f32>, ab: Option<f32>, aa: Option<f32>,
    r0: Option<f32>, g0: Option<f32>, b0: Option<f32>, a0: Option<f32>,
) {
    let ra = ra.unwrap_or(0.0);
    let ga = ga.unwrap_or(0.0);
    let ba = ba.unwrap_or(0.0);

    let r0 = r0.unwrap_or(0.0);
    let g0 = g0.unwrap_or(0.0);
    let b0 = b0.unwrap_or(0.0);
    let a0 = a0.unwrap_or(0.0);

    let ar = ar.unwrap_or(0.0);
    let ag = ag.unwrap_or(0.0);
    let ab = ab.unwrap_or(0.0);
    let aa = aa.unwrap_or(1.0);
    #[rustfmt::unskip]
    image.par_chunks_mut(4).for_each(|pixel| {
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;
        let a = pixel[3] as f32 / 255.0;

        let nr = rr * r + gr * g + br * b + ar * a + r0;
        let ng = rg * r + gg * g + bg * b + ag * a + g0;
        let nb = rb * r + gb * g + bb * b + ab * a + b0;
        let na = ra * r
                    + ga * g
                    + ba * b
                    + aa * a
                    + a0;

        let clamp = |x: f32| x.max(0.0).min(1.0);

        pixel[0] = (clamp(nr) * 255.0).round() as u8;
        pixel[1] = (clamp(ng) * 255.0).round() as u8;
        pixel[2] = (clamp(nb) * 255.0).round() as u8;
        pixel[3] = (clamp(na) * 255.0).round() as u8;
    });
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

pub fn flip_images(images: Vec<DynamicImage>, dir: Dirs) -> Result<Vec<DynamicImage>, Error> {
    zone!("flip_images");
    let images_out = images
        .into_par_iter()
        .map(|image| {
            zone!("flip_image");
            let mut new_image = image.clone().into_rgba8();
            flip(&mut new_image, dir);
            DynamicImage::ImageRgba8(new_image)
        })
        .collect();
    Ok(images_out)
}

pub fn turn_images(images: Vec<DynamicImage>, angle: f32) -> Result<Vec<DynamicImage>, Error> {
    zone!("turn_images");
    let images_out = images
        .into_par_iter()
        .map(|image| {
            zone!("turn_image");
            let mut new_image = image.clone().into_rgba8();
            turn(&mut new_image, angle);
            DynamicImage::ImageRgba8(new_image)
        })
        .collect();
    Ok(images_out)
}

pub fn shift_images(images: Vec<DynamicImage>, dir: Dirs, offset: i32, wrap: bool) -> Result<Vec<DynamicImage>, Error> {
    zone!("shift_images");
    let images_out = images
        .into_par_iter()
        .map(|image| {
            zone!("shift_image");
            let mut new_image = image.clone().into_rgba8();
            shift(&mut new_image, dir, offset, wrap);
            DynamicImage::ImageRgba8(new_image)
        })
        .collect();
    Ok(images_out)
}

#[rustfmt::skip]
pub fn map_images_colors(
    images: Vec<DynamicImage>,
    rr: f32, rg: f32, rb: f32, ra: Option<f32>,
    gr: f32, gg: f32, gb: f32, ga: Option<f32>,
    br: f32, bg: f32, bb: f32, ba: Option<f32>,
    ar: Option<f32>, ag: Option<f32>, ab: Option<f32>, aa: Option<f32>,
    r0: Option<f32>, g0: Option<f32>, b0: Option<f32>, a0: Option<f32>,
) -> Result<Vec<DynamicImage>, Error> {
    zone!("map_images_colors");
    let images_out = images
        .into_par_iter()
        .map(|image| {
            zone!("map_image_colors");
            let mut new_image = image.clone().into_rgba8();
            map_colors(
                &mut new_image,
                rr, rg, rb, ra,
                gr, gg, gb, ga,
                br, bg, bb, ba,
                ar, ag, ab, aa,
                r0, g0, b0, a0,
            );
            DynamicImage::ImageRgba8(new_image)
        })
        .collect();
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
    position: Option<(i32, i32)>,
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
            // Update delays
            let mut new_delays = image_data
                .delay
                .clone()
                .unwrap_or(vec![1.0; frames_out as usize]);
            let delay_diff = frames_out as i32 - new_delays.len() as i32;
            // Extend the number of delays to match frames by copying the first delay
            if delay_diff > 0 {
                new_delays.extend(vec![
                    *new_delays.get(0).unwrap_or(&1.0);
                    delay_diff as usize
                ]);
            } else if delay_diff < 0 {
                // sometimes DMIs can contain more delays than frames because they retain old data
                new_delays = new_delays[0..frames_out as usize].to_vec()
            }
            delay_out = Some(new_delays);
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
                match blend_icon(&mut new_image, &first_image, blend_mode, position) {
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
                match blend_icon(&mut new_image, &image2.into_rgba8(), blend_mode, position) {
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
            // Update delays
            let mut new_delays =
                base_icon_state
                    .delay
                    .clone()
                    .unwrap_or(vec![1.0; base_icon_state.frames as usize]);
            let delay_diff = base_icon_state.frames as i32 - new_delays.len() as i32;
            // Extend the number of delays to match frames by copying the first delay
            if delay_diff > 0 {
                new_delays.extend(vec![
                    *new_delays.get(0).unwrap_or(&1.0);
                    delay_diff as usize
                ]);
            } else if delay_diff < 0 {
                // sometimes DMIs can contain more delays than frames because they retain old data
                new_delays = new_delays[0..base_icon_state.frames as usize].to_vec()
            }
            base_icon_state.delay = Some(new_delays);
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
                match blend_icon(&mut new_image, &first_image, blend_mode, None) {
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
                match blend_icon(&mut new_image, &image2.into_rgba8(), blend_mode, None) {
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
            Transform::BlendIcon { icon, blend_mode, x, y } => {
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
                let position = Some((x.unwrap_or(1),y.unwrap_or(1)));
                let new_out = blend_images_other_universal(
                    image_data,
                    other_image_data.clone(),
                    &blending::BlendMode::from_u8(blend_mode)?,
                    position
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
            #[rustfmt::skip]
            Transform::MapColors {
                rr, rg, rb, ra,
                gr, gg, gb, ga,
                br, bg, bb, ba,
                ar, ag, ab, aa,
                r0, g0, b0, a0,
            } => {
                images = map_images_colors(
                    image_data.images.clone(),
                    rr.into_inner(), rg.into_inner(), rb.into_inner(), ra.map(OrderedFloat::into_inner),
                    gr.into_inner(), gg.into_inner(), gb.into_inner(), ga.map(OrderedFloat::into_inner),
                    br.into_inner(), bg.into_inner(), bb.into_inner(), ba.map(OrderedFloat::into_inner),
                    ar.map(OrderedFloat::into_inner), ag.map(OrderedFloat::into_inner), ab.map(OrderedFloat::into_inner), aa.map(OrderedFloat::into_inner),
                    r0.map(OrderedFloat::into_inner), g0.map(OrderedFloat::into_inner), b0.map(OrderedFloat::into_inner), a0.map(OrderedFloat::into_inner),
                )?;
            }
            Transform::Flip { dir } => {
                let dir = match dmi::dirs::Dirs::from_bits(*dir) {
                    Some(dir) => {
                        let image = image_data.images.first().unwrap();
                        if image.width() != image.height()
                            && !dmi::dirs::CARDINAL_DIRS.contains(&dir)
                        {
                            return Err(format!("Non-square icons cannot be flipped diagonally (Turned&Flipped)! (used Flip(dir={dir}) on {}x{})", image.width(), image.height()));
                        }
                        dir
                    }
                    None => return Err(format!("Invalid dir specified for Flip: {dir}")),
                };
                images = flip_images(image_data.images.clone(), dir)?;
            }
            Transform::Turn { angle } => {
                images = turn_images(image_data.images.clone(), angle.into_inner())?;
            }
            Transform::Shift { dir, offset, wrap} => {
                let dir = match dmi::dirs::Dirs::from_bits(*dir) {
                    Some(dir) => dir,
                    None => return Err(format!("Invalid dir specified for Shift: {dir}")),
                };
                images = shift_images(image_data.images.clone(), dir, *offset, *wrap != 0)?;
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
