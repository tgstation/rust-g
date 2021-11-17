#[macro_use]
mod byond;
#[allow(dead_code)]
mod error;

#[cfg(feature = "jobs")]
mod jobs;

#[cfg(feature = "cellularnoise")]
pub mod cellularnoise;
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
#[cfg(feature = "time")]
pub mod time;
#[cfg(feature = "toml")]
pub mod toml;
#[cfg(feature = "unzip")]
pub mod unzip;
#[cfg(feature = "url")]
pub mod url;
#[cfg(feature = "worleynoise")]
pub mod worleynoise;

#[cfg(not(target_pointer_width = "32"))]
compile_error!("rust-g must be compiled for a 32-bit target");
