use crate::error::{Error::SoundLen, Result};
use std::{collections::HashMap, fs::File};
use symphonia::{
    core::{
        codecs::audio::AudioDecoderOptions,
        formats::{FormatOptions, TrackType, probe::Hint},
        io::MediaSourceStream,
        meta::MetadataOptions,
        units::Timestamp,
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

    let fmt_opts: FormatOptions = Default::default();
    let meta_opts: MetadataOptions = Default::default();

    let mut format = match get_probe().probe(&hint, mss, fmt_opts, meta_opts) {
        Ok(r) => r,
        Err(e) => return Err(SoundLen(format!("Probe error: {e}"))),
    };

    match sound_length_simple(format.as_ref()) {
        Ok(r) => return Ok(format!("{:.3}", r as f32)),
        Err(_e) => (),
    };

    match sound_length_decode(format.as_mut()) {
        Ok(r) => Ok(format!("{:.3}", r as f32)),
        Err(e) => Err(e),
    }
}

fn sound_length_simple(format: &dyn symphonia::core::formats::FormatReader) -> Result<f64> {
    let track = match format.default_track(TrackType::Audio) {
        Some(r) => r,
        None => return Err(SoundLen("Could not get default track".to_string())),
    };

    let time_base = match track.time_base {
        Some(r) => r,
        None => return Err(SoundLen("Codec does not provide a time base.".to_string())),
    };

    let duration = match track.duration {
        Some(r) => r,
        None => return Err(SoundLen("Track does not provide duration".to_string())),
    };

    let duration_ts = duration
        .timestamp_from(Timestamp::ZERO)
        .ok_or_else(|| SoundLen("Track duration overflowed timestamp".to_string()))?;
    let time = time_base
        .calc_time(duration_ts)
        .ok_or_else(|| SoundLen("Could not convert track duration to time".to_string()))?;

    Ok(time.as_secs_f64() * 10.0)
}

fn sound_length_decode(format: &mut dyn symphonia::core::formats::FormatReader) -> Result<f64> {
    // Resolve track details and instantiate the decoder before packet iteration.
    let (track_id, samples_capacity, mut decoder) = {
        let track = match format.default_track(TrackType::Audio) {
            Some(r) => r,
            None => return Err(SoundLen("Could not get default track".to_string())),
        };

        let samples_capacity = track.num_frames.unwrap_or(0) as f64;

        let audio_codec_params = match track.codec_params.as_ref().and_then(|p| p.audio()) {
            Some(r) => r,
            None => return Err(SoundLen("Track has no audio codec parameters".to_string())),
        };

        let decoder_opts: AudioDecoderOptions = AudioDecoderOptions::default().gapless(true);
        let decoder = match get_codecs().make_audio_decoder(audio_codec_params, &decoder_opts) {
            Ok(r) => r,
            Err(e) => return Err(SoundLen(format!("Decoder creation error: {e}"))),
        };

        (track.id, samples_capacity, decoder)
    };

    // Read packets until we find one that belongs to the selected track.
    let encoded_packet = loop {
        match format.next_packet() {
            Ok(Some(packet)) => {
                if packet.track_id == track_id {
                    break packet;
                }
            }
            Ok(None) => return Err(SoundLen("No packets found for default track".to_string())),
            Err(e) => return Err(SoundLen(format!("Next_packet error: {e}"))),
        }
    };

    // Try to decode the data packet
    let decoded_packet = match decoder.decode(&encoded_packet) {
        Ok(r) => r,
        Err(e) => return Err(SoundLen(format!("Decode error: {e}"))),
    };

    // Grab the sample rate from the spec of the buffer.
    let sample_rate = decoded_packet.spec().rate() as f64;
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
