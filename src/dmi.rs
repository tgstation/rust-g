use std::fs::File;

use png::{Decoder, Encoder, HasParameters, OutputInfo};

use error::Result;

byond_fn! { dmi_strip_metadata(path) {
    strip_metadata(path).err()
} }

fn strip_metadata(path: &str) -> Result<()> {
    let (info, image) = read_png(path)?;
    write_png(path, info, image)?;

    Ok(())
}

fn read_png(path: &str) -> Result<(OutputInfo, Vec<u8>)> {
    let (info, mut reader) = Decoder::new(File::open(path)?).read_info()?;
    let mut buf = Vec::with_capacity(info.buffer_size());
    unsafe { buf.set_len(info.buffer_size()) }

    reader.next_frame(&mut buf)?;
    Ok((info, buf))
}

fn write_png(path: &str, info: OutputInfo, image: Vec<u8>) -> Result<()> {
    let mut encoder = Encoder::new(File::create(path)?, info.width, info.height);
    encoder.set(info.color_type).set(info.bit_depth);

    let mut writer = encoder.write_header()?;
    Ok(writer.write_image_data(&image)?)
}
