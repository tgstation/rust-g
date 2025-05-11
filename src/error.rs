use std::{
    io,
    num::{ParseFloatError, ParseIntError},
    result,
    str::Utf8Error,
};
use thiserror::Error;

#[cfg(feature = "http")]
use serde_json::Error as JsonError;
#[cfg(feature = "http")]
use ureq::Error as UreqError;

#[cfg(feature = "png")]
use image::error::ImageError;
#[cfg(feature = "png")]
use png::{DecodingError, EncodingError};

#[cfg(feature = "toml")]
use toml_dep::de::Error as TomlDeserializeError;
#[cfg(feature = "toml")]
use toml_dep::ser::Error as TomlSerializeError;

#[cfg(feature = "unzip")]
use zip::result::ZipError;

pub type Result<T> = result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Illegal null character in string.")]
    Null,
    #[error("Invalid UTF-8 character at position {position}.")]
    Utf8 { source: Utf8Error, position: usize },
    #[error("Invalid or empty filename specified.")]
    InvalidFilename,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Invalid algorithm specified.")]
    InvalidAlgorithm,
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloat(#[from] ParseFloatError),

    #[cfg(feature = "hash")]
    #[error("Unable to decode hex value.")]
    HexDecode,

    #[cfg(feature = "http")]
    #[error(transparent)]
    JsonSerialization(#[from] JsonError),
    #[cfg(feature = "http")]
    #[error(transparent)]
    Request(#[from] Box<UreqError>),
    #[cfg(feature = "http")]
    #[error("Unable to parse HTTP arguments: {0}")]
    HttpParse(String),
    #[cfg(feature = "http")]
    #[error("HTTP response over size limit")]
    HttpTooBig,

    #[cfg(feature = "iconforge")]
    #[error("IconForge error: {0}")]
    IconForge(String),

    #[cfg(feature = "png")]
    #[error(transparent)]
    ImageDecoding(#[from] DecodingError),
    #[cfg(feature = "png")]
    #[error(transparent)]
    ImageEncoding(#[from] EncodingError),
    #[cfg(feature = "png")]
    #[error(transparent)]
    GenericImage(#[from] ImageError),
    #[cfg(feature = "png")]
    #[error("Invalid png data.")]
    InvalidPngData,

    #[cfg(feature = "sound_len")]
    #[error("SoundLen error: {0}")]
    SoundLen(String),

    #[cfg(feature = "toml")]
    #[error(transparent)]
    TomlDeserialization(#[from] TomlDeserializeError),
    #[cfg(feature = "toml")]
    #[error(transparent)]
    TomlSerialization(#[from] TomlSerializeError),

    #[cfg(feature = "unzip")]
    #[error(transparent)]
    Unzip(#[from] ZipError),

    #[error("Panic during function execution: {0}")]
    Panic(String),
}

impl From<Utf8Error> for Error {
    fn from(source: Utf8Error) -> Self {
        Self::Utf8 {
            source,
            position: source.valid_up_to(),
        }
    }
}

impl From<Error> for String {
    fn from(error: Error) -> Self {
        error.to_string()
    }
}

impl From<Error> for Vec<u8> {
    fn from(error: Error) -> Self {
        error.to_string().into_bytes()
    }
}
