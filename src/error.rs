use std::{
    io,
    num::{ParseFloatError, ParseIntError},
    result,
    str::Utf8Error,
};
use thiserror::Error;

#[cfg(feature = "png")]
use png::{DecodingError, EncodingError};
#[cfg(feature = "png")]
use image::error::{ImageError};

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
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
    #[error(transparent)]
    GenericImageError(#[from] ImageError),
    #[cfg(feature = "png")]
    #[error("Invalid png data.")]
    InvalidPngDataError,
    #[cfg(feature = "http")]
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[cfg(feature = "http")]
    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),
    #[cfg(feature = "unzip")]
    #[error(transparent)]
    UnzipError(#[from] ZipError)
}

impl From<Utf8Error> for Error {
    fn from(source: Utf8Error) -> Error {
        Error::Utf8 {
            source,
            position: source.valid_up_to(),
        }
    }
}

impl From<Error> for String {
    fn from(error: Error) -> String {
        error.to_string()
    }
}

impl From<Error> for Vec<u8> {
    fn from(error: Error) -> Vec<u8> {
        error.to_string().into_bytes()
    }
}
