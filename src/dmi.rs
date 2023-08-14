use crate::error::{Error, Result};
use dmi::icon::Icon;
use dmi2svg::{dmi2svg, dmi2svg_symbol, dmi2svg_symbol_map};
use png::{Decoder, Encoder, OutputInfo, Reader};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::HashMap,
    fs::{create_dir_all, File, OpenOptions},
    io::{BufReader, Write},
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

byond_fn!(fn dmi_convert_to_svgcss(path, output, name_map) {
    convert_to_svgcss(path, output, name_map).err()
});

byond_fn!(fn dmi_start_svg_symbols(path) {
    start_svg_symbols(path).err()
});
byond_fn!(fn dmi_convert_to_svg_symbols(path, output, name_map) {
    convert_to_svg_symbols(path, output, name_map).err()
});
byond_fn!(fn dmi_end_svg_symbols(path) {
    end_svg_symbols(path).err()
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
    let reader = BufReader::new(File::open(path)?);
    let icon = Icon::load(reader).ok();
    if icon.is_none() {
        return Err(Error::InvalidPngData);
    }
    let states: Vec<_> = icon
        .unwrap()
        .states
        .iter()
        .map(|s| s.name.clone())
        .collect();
    Ok(serde_json::to_string(&states)?)
}

use base64::{engine::general_purpose, Engine as _};

/// Output is a ready-to-ship CSS file
fn convert_to_svgcss(path: &str, output: &str, name_map: &str) -> Result<()> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(Error::InvalidFilename);
    }

    let output_path = Path::new(output);

    let filename = path
        .file_stem()
        .ok_or(Error::InvalidFilename)?
        .to_string_lossy();

    let lookup: Option<HashMap<String, String>> = serde_json::from_str(name_map)?;

    let svgs = dmi2svg(path)?;
    let string = svgs
        .into_par_iter()
        .map(|state| {
            let name = {
                if let Some(table) = &lookup {
                    if let Some(name) = table.get(&state.name) {
                        name.as_str()
                    } else if state.name.is_empty() {
                        "DEFAULT"
                    } else {
                        state.name.as_str()
                    }
                } else if state.name.is_empty() {
                    "DEFAULT"
                } else {
                    state.name.as_str()
                }
            };
            let svg_b64 = general_purpose::STANDARD_NO_PAD.encode(state.svg);
            format!(
                ".{}.{}{{background-image: url(\"data:image/svg+xml;base64,{}\")}}\n",
                filename, name, svg_b64
            )
        })
        .collect::<String>();

    if let Some(fdir) = output_path.parent() {
        if !fdir.is_dir() {
            create_dir_all(fdir)?;
        }
    }

    let mut output = OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_path)?;

    writeln!(output, "{}", string)?;

    Ok(())
}

fn start_svg_symbols(path: &str) -> Result<()> {
    let path = Path::new(path);

    if let Some(fdir) = path.parent() {
        if !fdir.is_dir() {
            create_dir_all(fdir)?;
        }
    }

    std::fs::write(
        path,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="auto" height="auto" shape-rendering="crispEdges">"#,
    )?;

    Ok(())
}

/// Output is a file of raw svg <symbol>s split by newline
fn convert_to_svg_symbols(path: &str, output: &str, name_map: &str) -> Result<()> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(Error::InvalidFilename);
    }

    let output_path = Path::new(output);

    let lookup: Option<HashMap<String, String>> = serde_json::from_str(name_map)?;

    let symbols = if let Some(lookup) = lookup {
        dmi2svg_symbol_map(path, &lookup)?
    } else {
        dmi2svg_symbol(path)?
    };

    let mut output = OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_path)?;

    writeln!(output, "{}", symbols.join("\n"))?;

    Ok(())
}

fn end_svg_symbols(path: &str) -> Result<()> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(Error::InvalidFilename);
    }

    let mut output = OpenOptions::new().create(true).append(true).open(path)?;

    writeln!(output, "</svg>")?;

    Ok(())
}
