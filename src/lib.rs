#![warn(clippy::all, rust_2018_idioms)]

pub mod nbt;
pub mod snbt;
pub mod tree;
pub mod world;

use std::io::{self, BufRead, Read};
use flate2::bufread::GzDecoder;

/// Tries to read a gzip file from any type implementing `std::io::BufRead`, if it is not valid gzip, returns the `BufRead`
///
/// # Errors
///
/// This function will return an error if the provided data does not have a valid gzip header.
pub fn decompress_file<R: BufRead>(data: R) -> Result<GzDecoder<R>, R> {
    let gz = GzDecoder::new(data);

    if gz.header().is_some() { Ok(gz) } else { Err(gz.into_inner()) }
}

/// Parses an nbt file, from any type that implements `std::io::Read`.
///
/// # Errors
///
/// This function will return an error if the provided nbt data is invalid.
pub fn parse_nbt_file<R: Read>(data: &mut R, name: impl Into<String>) -> io::Result<nbt::RootTag> {
    nbt::RootTag::from_raw(data, name)
}
