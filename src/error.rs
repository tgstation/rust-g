use std::{
    fmt, io,
    num::{ParseFloatError, ParseIntError},
    result,
    str::Utf8Error,
};

#[cfg(feature = "png")]
use png::{DecodingError, EncodingError};
use std::fs::File;
use std::io::{BufWriter, IntoInnerError};

pub type Result<T> = result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Illegal null character in string.")]
    Null,
    #[fail(display = "Invalid UTF-8 character at position {}.", _1)]
    Utf8(#[cause] Utf8Error, usize),
    #[fail(display = "Invalid or empty filename specified.")]
    InvalidFilename,
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
    #[fail(display = "{}", _0)]
    Formatting(#[cause] fmt::Error),
    #[fail(display = "Invalid algorithm specified.")]
    InvalidAlgorithm,
    #[fail(display = "{}", _0)]
    InnerBufError(#[cause] IntoInnerError<BufWriter<File>>),
    #[cfg(feature = "png")]
    #[fail(display = "{}", _0)]
    ImageDecoding(#[cause] DecodingError),
    #[cfg(feature = "png")]
    #[fail(display = "{}", _0)]
    ImageEncoding(#[cause] EncodingError),
    #[fail(display = "{}", _0)]
    ParseIntError(#[cause] ParseIntError),
    #[fail(display = "{}", _0)]
    ParseFloatError(#[cause] ParseFloatError),
    #[cfg(feature = "png")]
    #[fail(display = "Invalid png data.")]
    InvalidPngDataError,
    #[cfg(feature = "http")]
    #[fail(display = "{}", _0)]
    RequestError(#[cause] reqwest::Error),
    #[cfg(feature = "http")]
    #[fail(display = "{}", _0)]
    SerializationError(#[cause] serde_json::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Error {
        Error::Utf8(error, error.valid_up_to())
    }
}

impl From<IntoInnerError<BufWriter<File>>> for Error {
    fn from(error: IntoInnerError<BufWriter<File>>) -> Error {
        Error::InnerBufError(error)
    }
}

impl From<fmt::Error> for Error {
    fn from(error: fmt::Error) -> Error {
        Error::Formatting(error)
    }
}

#[cfg(feature = "png")]
impl From<DecodingError> for Error {
    fn from(error: DecodingError) -> Error {
        Error::ImageDecoding(error)
    }
}

#[cfg(feature = "png")]
impl From<EncodingError> for Error {
    fn from(error: EncodingError) -> Error {
        Error::ImageEncoding(error)
    }
}

impl From<ParseIntError> for Error {
    fn from(error: ParseIntError) -> Error {
        Error::ParseIntError(error)
    }
}

impl From<ParseFloatError> for Error {
    fn from(error: ParseFloatError) -> Error {
        Error::ParseFloatError(error)
    }
}
#[cfg(feature = "http")]
impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Error {
        Error::RequestError(error)
    }
}

#[cfg(feature = "http")]
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::SerializationError(error)
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
