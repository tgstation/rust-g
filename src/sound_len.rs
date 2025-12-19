use crate::error::{Error::SoundLen, Result};
use std::{collections::HashMap, fs::File, time::Duration};
use symphonia::{
    self,
    core::{
        codecs::DecoderOptions,
        formats::FormatOptions,
        io::MediaSourceStream,
        meta::MetadataOptions,
        probe::{Hint, ProbeResult},
    },
    default::{get_codecs, get_probe},
};

byond_fn!(fn sound_len(sound_path) {
    match get_sound_length(sound_path) {
        Ok(r) => return Some(r),
        Err(e) => return Some(e.to_string())
    }
});

fn get_sound_length(sound_path: &str) -> Result<String> {
    // Try to open the file
    let sound_src = match File::open(sound_path) {
        Ok(r) => r,
        Err(e) => return Err(SoundLen(format!("Couldn't open file, {e}"))),
    };

    // Audio probe things
    let mss = MediaSourceStream::new(Box::new(sound_src), Default::default());

    let mut hint = Hint::new();
    hint.with_extension("ogg");
    hint.with_extension("mp3");

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = symphonia::core::formats::FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };

    let probed = match get_probe().format(&hint, mss, &fmt_opts, &meta_opts) {
        Ok(r) => r,
        Err(e) => return Err(SoundLen(format!("Probe error: {e}"))),
    };

    match sound_length_simple(&probed) {
        Ok(r) => return Ok(format!("{:.3}", r as f32)),
        Err(_e) => (),
    };

    match sound_length_decode(probed) {
        Ok(r) => Ok(format!("{:.3}", r as f32)),
        Err(e) => Err(e),
    }
}

fn sound_length_simple(probed: &ProbeResult) -> Result<f64> {
    let format = &probed.format;

    let track = match format.default_track() {
        Some(r) => r,
        None => return Err(SoundLen("Could not get default track".to_string())),
    };

    let time_base = match track.codec_params.time_base {
        Some(r) => r,
        None => return Err(SoundLen("Codec does not provide a time base.".to_string())),
    };

    let n_frames = match track.codec_params.n_frames {
        Some(r) => r,
        None => return Err(SoundLen("Codec does not provide frame count".to_string())),
    };

    let time = time_base.calc_time(n_frames);
    let duration = Duration::from_secs(time.seconds) + Duration::from_secs_f64(time.frac);

    Ok(duration.as_secs_f64() * 10.0)
}

fn sound_length_decode(probed: ProbeResult) -> Result<f64> {
    let mut format = probed.format;

    let track = match format.default_track() {
        Some(r) => r,
        None => return Err(SoundLen("Could not get default track".to_string())),
    };

    // Grab the number of frames of the track
    let samples_capacity = if let Some(n_frames) = track.codec_params.n_frames {
        n_frames as f64
    } else {
        0.0
    };

    // Create a decoder using the provided codec parameters in the track.
    let decoder_opts: DecoderOptions = Default::default();
    let mut decoder = match get_codecs().make(&track.codec_params, &decoder_opts) {
        Ok(r) => r,
        Err(e) => return Err(SoundLen(format!("Decoder creation error: {e}"))),
    };

    // Try to grab a data packet from the container
    let encoded_packet = match format.next_packet() {
        Ok(r) => r,
        Err(e) => return Err(SoundLen(format!("Next_packet error: {e}"))),
    };

    // Try to decode the data packet
    let decoded_packet = match decoder.decode(&encoded_packet) {
        Ok(r) => r,
        Err(e) => return Err(SoundLen(format!("Decode error: {e}"))),
    };

    // Grab the sample rate from the spec of the buffer.
    let sample_rate = decoded_packet.spec().rate as f64;
    // Math!
    let duration_in_desciseconds = samples_capacity / sample_rate * 10.0;
    Ok(duration_in_desciseconds)
}

byond_fn!(
    fn sound_len_list(list) {
        Some(get_sound_length_list(list))
    }
);

fn get_sound_length_list(list: &str) -> String {
    let json: Vec<&str> = match serde_json::from_str(list) {
        Ok(r) => r,
        Err(_e) => return String::from("Fatal error: Bad json"),
    };

    let mut successes = HashMap::new();
    let mut errors = HashMap::new();

    for path_string in json.iter() {
        match get_sound_length(path_string) {
            Ok(r) => successes.insert(path_string.to_string(), r),
            Err(e) => errors.insert(path_string.to_string(), e.to_string()),
        };
    }

    let mut out = HashMap::new();
    out.insert("successes".to_string(), successes);
    out.insert("errors".to_string(), errors);

    serde_json::to_string(&out).unwrap_or_else(|_| "{}".to_owned())
}
