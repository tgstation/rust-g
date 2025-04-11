use crate::error::{Error, Result};
use dmi::icon::{DmiVersion, Icon, IconState, Looping};
use image::{DynamicImage, GenericImage, Pixel, Rgba, RgbaImage};
use png::{Decoder, Encoder, OutputInfo, Reader};
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use std::{
    fs::{create_dir_all, File},
    io::BufReader,
    num::NonZeroU32,
    path::Path,
};

byond_fn!(fn dmi_strip_metadata(path) {
    strip_metadata(path).err()
});

byond_fn!(fn dmi_create_png(path, width, height, data) {
    create_png(path, width, height, data).err()
});

byond_fn!(fn dmi_resize_png(path, width, height, resizetype) {
    let resizetype = match resizetype {
        "catmull" => image::imageops::CatmullRom,
        "gaussian" => image::imageops::Gaussian,
        "lanczos3" => image::imageops::Lanczos3,
        "nearest" => image::imageops::Nearest,
        "triangle" => image::imageops::Triangle,
        _ => image::imageops::Nearest,
    };
    resize_png(path, width, height, resizetype).err()
});

byond_fn!(fn dmi_icon_states(path) {
    read_states(path).ok()
});

fn strip_metadata(path: &str) -> Result<()> {
    let (reader, frame_info, image) = read_png(path)?;
    write_png(path, &reader, &frame_info, &image, true)
}

fn read_png(path: &str) -> Result<(Reader<File>, OutputInfo, Vec<u8>)> {
    let mut reader = Decoder::new(File::open(path)?).read_info()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let frame_info = reader.next_frame(&mut buf)?;

    Ok((reader, frame_info, buf))
}

fn write_png(
    path: &str,
    reader: &Reader<File>,
    info: &OutputInfo,
    image: &[u8],
    strip: bool,
) -> Result<()> {
    let mut encoder = Encoder::new(File::create(path)?, info.width, info.height);
    encoder.set_color(info.color_type);
    encoder.set_depth(info.bit_depth);

    let reader_info = reader.info();
    if let Some(palette) = reader_info.palette.clone() {
        encoder.set_palette(palette);
    }

    if let Some(trns_chunk) = reader_info.trns.clone() {
        encoder.set_trns(trns_chunk);
    }

    let mut writer = encoder.write_header()?;
    // Handles zTxt chunk copying from the original image if we /don't/ want to strip it
    if !strip {
        for chunk in &reader_info.compressed_latin1_text {
            writer.write_text_chunk(chunk)?;
        }
    }
    Ok(writer.write_image_data(image)?)
}

fn create_png(path: &str, width: &str, height: &str, data: &str) -> Result<()> {
    let width = width.parse::<u32>()?;
    let height = height.parse::<u32>()?;

    let bytes = data.as_bytes();
    if bytes.len() % 7 != 0 {
        return Err(Error::InvalidPngData);
    }

    let mut result: Vec<u8> = Vec::new();
    for pixel in bytes.chunks_exact(7) {
        for channel in pixel[1..].chunks_exact(2) {
            result.push(u8::from_str_radix(std::str::from_utf8(channel)?, 16)?);
        }
    }

    if let Some(fdir) = Path::new(path).parent() {
        if !fdir.is_dir() {
            create_dir_all(fdir)?;
        }
    }

    let mut encoder = Encoder::new(File::create(path)?, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    Ok(writer.write_image_data(&result)?)
}

fn resize_png<P: AsRef<Path>>(
    path: P,
    width: &str,
    height: &str,
    resizetype: image::imageops::FilterType,
) -> std::result::Result<(), Error> {
    let width = width.parse::<u32>()?;
    let height = height.parse::<u32>()?;

    let img = image::open(path.as_ref())?;

    let newimg = img.resize(width, height, resizetype);

    Ok(newimg.save_with_format(path.as_ref(), image::ImageFormat::Png)?)
}

/// Output is a JSON string for reading within BYOND
///
/// Erroring at any point will produce an empty string
fn read_states(path: &str) -> Result<String> {
    let file = File::open(path).map(BufReader::new)?;
    let decoder = png::Decoder::new(file);
    let reader = decoder.read_info().map_err(|_| Error::InvalidPngData)?;
    let info = reader.info();
    let mut states = Vec::<String>::new();
    for ztxt in &info.compressed_latin1_text {
        let text = ztxt.get_text()?;
        text.lines()
            .take_while(|line| !line.contains("# END DMI"))
            .filter_map(|line| {
                line.trim()
                    .strip_prefix("state = \"")
                    .and_then(|line| line.strip_suffix('"'))
            })
            .for_each(|state| {
                states.push(state.to_owned());
            });
    }
    Ok(serde_json::to_string(&states)?)
}

byond_fn!(fn dmi_flatten_layers(data) {
    flatten_layers(data).ok()
});

fn flatten_layers(data: &str) -> Result<String> {
    let layers: Vec<Vec<Option<String>>> = serde_json::from_str(data)?;
    let mut out_layer: Vec<Rgba<u8>> = vec![];
    out_layer.resize(
        layers.iter().fold(0, |max, layer| max.max(layer.len())),
        Rgba::<u8>([0, 0, 0, 0]),
    );
    for layer in layers {
        for (i, pixel_opt) in layer.iter().enumerate() {
            if let Some(pixel) = pixel_opt {
                let mut samples: Vec<u8> = vec![];
                let pixel_bytes: Vec<u8> = pixel.bytes().skip(1).collect();
                for chunk in pixel_bytes.as_slice().chunks_exact(2) {
                    samples.push(u8::from_str_radix(std::str::from_utf8(chunk)?, 16)?);
                }
                samples.resize(4, 0);
                out_layer[i].blend(Rgba::from_slice(samples.as_slice()));
            }
        }
    }
    Ok(serde_json::to_string(
        &out_layer
            .iter()
            .map(|Rgba([r, g, b, a])| {
                if *a < u8::MAX {
                    format!("#{r:02X?}{g:02X?}{b:02X?}{a:02X?}")
                } else {
                    format!("#{r:02X?}{g:02X?}{b:02X?}")
                }
            })
            .collect::<Vec<String>>(),
    )?)
}

#[derive(Deserialize_repr, Clone, Copy)]
#[repr(u8)]
enum DmiBuldStateDirCount {
    One = 1,
    Four = 4,
    Eight = 8,
}

#[derive(Deserialize)]
struct DmiBuildState {
    name: String,
    dirs: DmiBuldStateDirCount,
    #[serde(default)]
    delay: Option<Vec<f32>>,
    #[serde(default)]
    rewind: Option<u8>,
    #[serde(default)]
    movement: Option<u8>,
    #[serde(default)]
    loop_count: Option<NonZeroU32>,
    pixels: Vec<Option<String>>,
}

#[derive(Deserialize)]
struct DmiBuildParams {
    width: u32,
    height: u32,
    states: Vec<DmiBuildState>,
}

byond_fn!(fn dmi_create_dmi(out_path, params_string) {
    create_dmi(out_path, params_string).err()
});

fn create_dmi(out_path: &str, params_string: &str) -> Result<()> {
    let DmiBuildParams {
        width,
        height,
        states,
    } = serde_json::from_str(params_string)?;
    let sprite_size = (width * height) as usize;
    if sprite_size == 0 {
        return Err(Error::ZeroIconSize);
    }
    if states.is_empty() {
        return Err(Error::NoDmiStates);
    };
    let mut out_icon = Icon {
        version: DmiVersion::default(),
        width,
        height,
        states: vec![],
    };
    for DmiBuildState {
        name,
        dirs,
        delay,
        loop_count,
        rewind,
        movement,
        pixels,
    } in states
    {
        let frame_count = delay.as_ref().map_or(1, |delays| delays.len());
        let pixel_count = pixels.len();
        let expected_pixel_count = frame_count * sprite_size;
        if pixel_count != expected_pixel_count {
            return Err(Error::StateDataFormat(
                name,
                format!("Expected {expected_pixel_count} pixels, got {pixel_count}"),
            ));
        };
        let mut out_state = IconState {
            name: name.clone(),
            dirs: dirs as u8,
            frames: frame_count as u32,
            images: vec![],
            delay,
            loop_flag: loop_count.map_or(Looping::Indefinitely, Looping::NTimes),
            rewind: rewind.is_some_and(|r| r != 0),
            movement: movement.is_some_and(|m| m != 0),
            hotspot: None,
            unknown_settings: None,
        };
        for chunk in pixels.chunks_exact(sprite_size) {
            let mut image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
                width,
                height,
                Rgba([192, 192, 192, 0]),
            ));
            for (i, pixel_opt) in chunk.iter().enumerate() {
                if let Some(pixel) = pixel_opt {
                    let mut samples: Vec<u8> = vec![];
                    let pixel_bytes: Vec<u8> = pixel.bytes().skip(1).collect();
                    for chunk in pixel_bytes.as_slice().chunks_exact(2) {
                        samples.push(u8::from_str_radix(std::str::from_utf8(chunk)?, 16)?);
                    }
                    match samples.len() {
                        3 => samples.push(u8::MAX),
                        4 if samples[3] == 0 => continue,
                        4 => (),
                        _ => {
                            return Err(Error::StateDataFormat(
                                name,
                                format!("Invalid color string \"{pixel}\" at index {i} (must be an RGB or RGBA hex color string)"),
                            ))
                        }
                    }
                    let x = i as u32 % width;
                    let y = i as u32 / width;
                    image.put_pixel(x, y, Rgba([samples[0], samples[1], samples[2], samples[3]]));
                }
            }
            out_state.images.push(image);
        }
        out_icon.states.push(out_state);
    }
    out_icon.save(&mut File::create(out_path)?)?;
    Ok(())
}
