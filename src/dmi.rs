use crate::error::{Error, Result};
use png::{Decoder, Encoder, OutputInfo};
use std::{
    fs::{create_dir_all, File},
    path::Path,
};

byond_fn! { dmi_strip_metadata(path) {
    strip_metadata(path).err()
} }

byond_fn! { dmi_create_png(path, width, height, data) {
    create_png(path, width, height, data).err()
} }

byond_fn! { dmi_resize_png(path, width, height, resizetype) {
    let resizetype = match resizetype {
        "catmull" => image::imageops::CatmullRom,
        "gaussian" => image::imageops::Gaussian,
        "lanczos3" => image::imageops::Lanczos3,
        "nearest" => image::imageops::Nearest,
        "triangle" => image::imageops::Triangle,
        _ => image::imageops::Nearest,
    };
    resize_png(path, width, height, resizetype).err()
} }

fn strip_metadata(path: &str) -> Result<()> {
    let (info, image) = read_png(path)?;
    Ok(write_png(path, info, image)?)
}

fn read_png(path: &str) -> Result<(OutputInfo, Vec<u8>)> {
    let (info, mut reader) = Decoder::new(File::open(path)?).read_info()?;
    let mut buf = vec![0; info.buffer_size()];

    reader.next_frame(&mut buf)?;
    Ok((info, buf))
}

fn write_png(path: &str, info: OutputInfo, image: Vec<u8>) -> Result<()> {
    let mut encoder = Encoder::new(File::create(path)?, info.width, info.height);
    encoder.set_color(info.color_type);
    encoder.set_depth(info.bit_depth);

    let mut writer = encoder.write_header()?;
    Ok(writer.write_image_data(&image)?)
}

fn create_png(path: &str, width: &str, height: &str, data: &str) -> Result<()> {
    let width = width.parse::<u32>()?;
    let height = height.parse::<u32>()?;

    let bytes = data.as_bytes();
    if bytes.len() % 7 != 0 {
        return Err(Error::InvalidPngDataError);
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
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    Ok(writer.write_image_data(&result)?)
}

fn resize_png<P: AsRef<Path>>(path: P, width: &str, height: &str, resizetype: image::imageops::FilterType) -> std::result::Result<(), Error> {
    let width = width.parse::<u32>()?;
    let height = height.parse::<u32>()?;

    let img = image::open(path.as_ref())?;

    let newimg = img.resize(width, height, resizetype);

    Ok(newimg.save_with_format(path.as_ref(), image::ImageFormat::Png)?)
}
