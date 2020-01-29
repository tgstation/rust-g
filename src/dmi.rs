use std::fs::File;
use png::{Decoder, Encoder, HasParameters, OutputInfo};

use error::{Result, Error};

byond_fn! { dmi_strip_metadata(path) {
    strip_metadata(path).err()
} }

byond_fn! { dmi_create_png(path, width, height , data) {
    create_png(path, width, height , data).err()
}}

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
    let width = u32::from_str_radix(width,10)?;
    let height = u32::from_str_radix(height,10)?;

    let mut result : Vec<u8> = Vec::new();
    let mut str_iter = data.chars().peekable();
    while str_iter.peek().is_some(){
        let single: String = str_iter.by_ref().take(7).collect();
        if single.chars().count() != 7{
            return Err(Error::InvalidPngDataError);
        }
        let r = u8::from_str_radix(&single[1..3], 16)?;
        let g = u8::from_str_radix(&single[3..5], 16)?;
        let b = u8::from_str_radix(&single[5..7], 16)?;
        result.push(r);
        result.push(g);
        result.push(b);
    }

    let mut encoder = Encoder::new(File::create(path)?, width, height);
    encoder.set(png::ColorType::RGB);
    encoder.set(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    Ok(writer.write_image_data(&result)?)
}
