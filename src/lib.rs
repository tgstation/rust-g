#[macro_use]
extern crate failure;

#[cfg(feature="chrono")]
extern crate chrono;
#[cfg(feature="crypto-hash")]
extern crate crypto_hash;
#[cfg(feature="git2")]
extern crate git2;
#[cfg(feature="hex")]
extern crate hex;
#[cfg(feature="percent-encoding")]
extern crate percent_encoding;
#[cfg(feature="png")]
extern crate png;
#[cfg(feature="http")]
extern crate reqwest;
#[cfg(feature="http")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature="http")]
extern crate serde_json;
#[cfg(feature="http")]
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod byond;
#[allow(dead_code)]
mod error;
mod jobs;

#[cfg(feature="dmi")]
pub mod dmi;
#[cfg(feature="file")]
pub mod file;
#[cfg(feature="git")]
pub mod git;
#[cfg(feature="hash")]
pub mod hash;
#[cfg(feature="log")]
pub mod log;
#[cfg(feature="url")]
pub mod url;
#[cfg(feature="http")]
pub mod http;
