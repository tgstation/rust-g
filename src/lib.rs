#[macro_use]
extern crate failure;

#[cfg(feature = "chrono")]
extern crate chrono;
#[cfg(feature = "crypto-hash")]
extern crate crypto_hash;
#[cfg(feature = "git2")]
extern crate git2;
#[cfg(feature = "hex")]
extern crate hex;
#[cfg(feature = "noise")]
extern crate noise;
#[cfg(feature = "percent-encoding")]
extern crate percent_encoding;
#[cfg(feature = "png")]
extern crate png;
#[cfg(feature = "http")]
extern crate reqwest;
#[cfg(any(feature = "http", feature = "sql"))]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "sql")]
extern crate mysql;
#[cfg(any(feature = "http", feature = "sql"))]
extern crate serde_json;

#[macro_use]
mod byond;
#[allow(dead_code)]
mod error;
mod jobs;

#[cfg(feature = "dmi")]
pub mod dmi;
#[cfg(feature = "file")]
pub mod file;
#[cfg(feature = "git")]
pub mod git;
#[cfg(feature = "hash")]
pub mod hash;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "log")]
pub mod log;
#[cfg(feature = "noise")]
pub mod noise_gen;
#[cfg(feature = "sql")]
pub mod sql;
#[cfg(feature = "url")]
pub mod url;

#[cfg(not(target_pointer_width = "32"))]
compile_error!("rust-g must be compiled for a 32-bit target");
