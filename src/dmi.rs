use std::fs::File;
use png::{Decoder, Encoder, HasParameters};

use error::Result;

byond_fn! { dmi_strip_metadata(path) {
    strip_metadata(path).err()
} }

fn strip_metadata(path: &str) -> Result<()> {
    // Read in the PNG, discarding metadata
    let (info, mut reader) = Decoder::new(File::open(path)?).read_info()?;
    let mut buffer = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut buffer)?;
    drop(reader);

    // Write the PNG back out
    let mut encoder = Encoder::new(File::create(path)?, info.width, info.height);
    encoder.set(info.color_type);
    encoder.set(info.bit_depth);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&buffer)?;
    Ok(())
}
