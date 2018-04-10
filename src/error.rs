use std::io;
use std::result;
use std::str::Utf8Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Illegal null character in string.")]
    Null,
    #[fail(display = "Invalid UTF-8 character at position {}.", _1)]
    Utf8(#[cause] Utf8Error, usize),
    #[fail(display = "Invalid or empty filename specified.")]
    InvalidName,
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
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
