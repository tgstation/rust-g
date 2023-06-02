use std::{
    io,
    num::{ParseFloatError, ParseIntError},
    result,
    str::Utf8Error,
};
use thiserror::Error;

#[cfg(feature = "png")]
use image::error::ImageError;
#[cfg(feature = "png")]
use png::{DecodingError, EncodingError};

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
    #[cfg(feature = "png")]
    #[error(transparent)]
    ImageDecoding(#[from] DecodingError),
    #[cfg(feature = "png")]
    #[error(transparent)]
    ImageEncoding(#[from] EncodingError),
    #[cfg(feature = "http")]
    #[error(transparent)]
    JsonSerialization(#[from] serde_json::Error),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloat(#[from] ParseFloatError),
    #[error(transparent)]
    GenericImage(#[from] ImageError),
    #[cfg(feature = "png")]
    #[error("Invalid png data.")]
    InvalidPngData,
    #[cfg(feature = "http")]
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[cfg(feature = "toml")]
    #[error(transparent)]
    TomlDeserialization(#[from] toml_dep::de::Error),
    #[cfg(feature = "toml")]
    #[error(transparent)]
    TomlSerialization(#[from] toml_dep::ser::Error),
    #[cfg(feature = "unzip")]
    #[error(transparent)]
    Unzip(#[from] ZipError),
    #[cfg(feature = "hash")]
    #[error("Unable to decode hex value.")]
    HexDecode,
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
