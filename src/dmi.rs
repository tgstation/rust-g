use crate::error::{Error, Result};
use image::Rgba;
use qrcode::{render::svg, QrCode};
use dmi::{
    error::DmiError,
    icon::{Icon, Looping},
};
use png::{text_metadata::ZTXtChunk, Decoder, Encoder, OutputInfo, Reader};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{
    fmt::Write,
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

byond_fn!(fn dmi_read_metadata(path) {
    match read_metadata(path) {
        Ok(metadata) => Some(metadata),
        Err(error) => Some(serde_json::to_string(&error.to_string()).unwrap()),
    }
});

byond_fn!(fn dmi_inject_metadata(path, metadata) {
    inject_metadata(path, metadata).err()
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

    let mut result: Vec<u8> = Vec::new();
    for pixel in bytes.split(|&b| b == b'#').skip(1) {
        if pixel.len() != 6 && pixel.len() != 8 {
            return Err(Error::InvalidPngData);
        }
        for channel in pixel.chunks_exact(2) {
            result.push(u8::from_str_radix(std::str::from_utf8(channel)?, 16)?);
        }
        // If only RGB is provided for any pixel we also add alpha
        if pixel.len() == 6 {
            result.push(255);
        }
    }

    if let Some(fdir) = Path::new(path).parent() {
        if !fdir.is_dir() {
            create_dir_all(fdir)?;
        }
    }

    let mut encoder = Encoder::new(File::create(path)?, width, height);
    encoder.set_color(png::ColorType::Rgba);
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

#[derive(Serialize_repr, Deserialize_repr, Clone, Copy)]
#[repr(u8)]
enum DmiStateDirCount {
    One = 1,
    Four = 4,
    Eight = 8,
}

impl TryFrom<u8> for DmiStateDirCount {
    type Error = u8;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            4 => Ok(Self::Four),
            8 => Ok(Self::Eight),
            n => Err(n),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct DmiState {
    name: String,
    dirs: DmiStateDirCount,
    #[serde(default)]
    delay: Option<Vec<f32>>,
    #[serde(default)]
    rewind: Option<u8>,
    #[serde(default)]
    movement: Option<u8>,
    #[serde(default)]
    loop_count: Option<NonZeroU32>,
    #[serde(default)]
    hotspot: Option<(u32, u32, u32)>,
}

#[derive(Serialize, Deserialize)]
struct DmiMetadata {
    width: u32,
    height: u32,
    states: Vec<DmiState>,
}

fn read_metadata(path: &str) -> Result<String> {
    let dmi = Icon::load(File::open(path).map(BufReader::new)?)?;
    let metadata = DmiMetadata {
        width: dmi.width,
        height: dmi.height,
        states: dmi
            .states
            .iter()
            .map(|state| {
                Ok(DmiState {
                    name: state.name.clone(),
                    dirs: DmiStateDirCount::try_from(state.dirs).map_err(|n| {
                        DmiError::IconState(format!(
                            "State \"{}\" has invalid dir count (expected 1, 4, or 8, got {})",
                            state.name, n
                        ))
                    })?,
                    delay: state.delay.clone(),
                    movement: state.movement.then_some(1),
                    rewind: state.rewind.then_some(1),
                    loop_count: match state.loop_flag {
                        Looping::Indefinitely => None,
                        Looping::NTimes(n) => Some(n),
                    },
                    hotspot: state.hotspot.map(|hotspot| (hotspot.x, hotspot.y, 1)),
                })
            })
            .collect::<Result<Vec<DmiState>>>()?,
    };
    Ok(serde_json::to_string(&metadata)?)
}

fn inject_metadata(path: &str, metadata: &str) -> Result<()> {
    let read_file = File::open(path).map(BufReader::new)?;
    let decoder = png::Decoder::new(read_file);
    let mut reader = decoder.read_info().map_err(|_| Error::InvalidPngData)?;
    let new_dmi_metadata: DmiMetadata = serde_json::from_str(metadata)?;
    let mut new_metadata_string = String::new();
    writeln!(new_metadata_string, "# BEGIN DMI")?;
    writeln!(new_metadata_string, "version = 4.0")?;
    writeln!(new_metadata_string, "\twidth = {}", new_dmi_metadata.width)?;
    writeln!(
        new_metadata_string,
        "\theight = {}",
        new_dmi_metadata.height
    )?;
    for state in new_dmi_metadata.states {
        writeln!(new_metadata_string, "state = \"{}\"", state.name)?;
        writeln!(new_metadata_string, "\tdirs = {}", state.dirs as u8)?;
        writeln!(
            new_metadata_string,
            "\tframes = {}",
            state.delay.as_ref().map_or(1, Vec::len)
        )?;
        if let Some(delay) = state.delay {
            writeln!(
                new_metadata_string,
                "\tdelay = {}",
                delay
                    .iter()
                    .map(f32::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            )?;
        }
        if state.rewind.is_some_and(|r| r != 0) {
            writeln!(new_metadata_string, "\trewind = 1")?;
        }
        if state.movement.is_some_and(|m| m != 0) {
            writeln!(new_metadata_string, "\tmovement = 1")?;
        }
        if let Some(loop_count) = state.loop_count {
            writeln!(new_metadata_string, "\tloop = {loop_count}")?;
        }
        if let Some((hotspot_x, hotspot_y, hotspot_frame)) = state.hotspot {
            writeln!(
                new_metadata_string,
                "\totspot = {hotspot_x},{hotspot_y},{hotspot_frame}"
            )?;
        }
    }
    writeln!(new_metadata_string, "# END DMI")?;
    let mut info = reader.info().clone();
    info.compressed_latin1_text
        .push(ZTXtChunk::new("Description", new_metadata_string));
    let mut raw_image_data: Vec<u8> = vec![];
    while let Some(row) = reader.next_row()? {
        raw_image_data.append(&mut row.data().to_vec());
    }
    let encoder = png::Encoder::with_info(File::create(path)?, info)?;
    encoder.write_header()?.write_image_data(&raw_image_data)?;
    Ok(())
}

byond_fn!(fn create_qr_code_png(path, data) {
    let code = match QrCode::new(data.as_bytes()) {
        Ok(code) => code,
        Err(err) => return Some(format!("Error: Could not read data into QR code: {err}"))
    };
    let image = code.render::<Rgba<u8>>().build();
    match image.save(path) {
        Ok(_) => Some(String::from(path)),
        Err(err) => Some(format!("Error: Could not write QR code image to path: {err}"))
    }
});

byond_fn!(fn create_qr_code_svg(data) {
    let code = match QrCode::new(data.as_bytes()) {
        Ok(code) => code,
        Err(err) => return Some(format!("Error: Could not read data into QR code: {err}"))
    };
    let svg_xml = code.render::<svg::Color>().build();
    Some(svg_xml)
});
