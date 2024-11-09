use crate::error::Result;
use symphonia::{
    self,
    core::{
        codecs::DecoderOptions,
        formats::FormatOptions,
        io::MediaSourceStream,
        meta::MetadataOptions,
        probe::Hint
    },
    default::{get_codecs, get_probe},
};
use std::{
    fs::File,
    path::Path,
};

byond_fn!(fn sound_len(sound_path) {
    match get_sound_length(sound_path) {
        Ok(r) => return Some(r),
        Err(e) => return Some(e.to_string())
    }
});

fn get_sound_length (sound_path: &str) -> Result<String> {
    let path = Path::new(sound_path);

    // Gracefully exit if the filepath is invalid.
    if !path.exists() {
        return Ok(String::from("path doesnt exist!"));
    }

    // Try to open the file
    let sound_src = match File::open(&path) {
        Ok(r) => r,
        Err(_e) => return Ok(String::from("Couldn't open file!")),
    };


    // Audio probe things
    let mss = MediaSourceStream::new(Box::new(sound_src), Default::default());

    let mut hint = Hint::new();
    hint.with_extension("ogg");

    let meta_opts: MetadataOptions = Default::default();
    let mut fmt_opts: FormatOptions = Default::default();
    fmt_opts.enable_gapless = true;

    let probed = match get_probe().format(&hint, mss, &fmt_opts, &meta_opts) {
        Ok(r) => r,
        Err(_e) => return Ok(String::from("Failed to probe file!")),
    };

    let mut format = probed.format;

    let track = match format.default_track() {
        Some(r) => r,
        None => return Ok(String::from("Failed to grab track from container!")),
    };

    // Grab the number of frames of the track
    let samples_capacity: f64 = if let Some(n_frames) = track.codec_params.n_frames {
        n_frames as f64
    } else {
        0.0
    };

    // Create a decoder using the provided codec parameters in the track.
    let decoder_opts: DecoderOptions = Default::default();
    let mut decoder = match get_codecs().make(&track.codec_params, &decoder_opts) {
        Ok(r) => r,
        Err(_e) => return Ok(String::from("Failed to generate decoder!")),
    };

    // Try to grab a data packet from the container
    let encoded_packet = match format.next_packet() {
        Ok(r) => r,
        Err(_e) => return Ok(String::from("Failed to grab packet from container!")),
    };

    // Try to decode the data packet
    let decoded_packet = match decoder.decode(&encoded_packet) {
        Ok(r) => r,
        Err(_e) => return Ok(String::from("Failed to decode packet!"))
    };

    // Grab the sample rate from the spec of the buffer.
    let sample_rate: f64 = decoded_packet.spec().rate as f64;
    // Math!
    let duration_in_seconds: f32 = (samples_capacity / sample_rate) as f32;
    return Ok(duration_in_seconds.to_string());

}
