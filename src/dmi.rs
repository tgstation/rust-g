use std::fs::{File, create_dir_all};
use std::path::Path;
use png::{Decoder, Encoder, HasParameters, OutputInfo};

use error::{Result, Error};

byond_fn! { dmi_strip_metadata(path) {
    strip_metadata(path).err()
} }

byond_fn! { dmi_create_png(path, width, height, data) {
    create_png(path, width, height , data).err()
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
    encoder.set(info.color_type).set(info.bit_depth);

    let mut writer = encoder.write_header()?;
    Ok(writer.write_image_data(&image)?)
}

fn create_png(path: &str, width: &str, height: &str, data: &str) -> Result<()> {
    let width = u32::from_str_radix(width, 10)?;
    let height = u32::from_str_radix(height, 10)?;

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
    encoder.set(png::ColorType::RGB);
    encoder.set(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    Ok(writer.write_image_data(&result)?)
}
