#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::App;

mod nbt;
pub use nbt::RootTag;

mod snbt;

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
pub fn parse_nbt_file<R: Read>(data: &mut R) -> io::Result<RootTag> {
    RootTag::from_raw(data)
}
