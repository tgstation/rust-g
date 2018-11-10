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
