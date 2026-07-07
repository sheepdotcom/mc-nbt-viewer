#![warn(clippy::all, rust_2018_idioms)]

mod app;
use std::io::{self, Read};

pub use app::App;
use flate2::read::GzDecoder;

pub fn decompress_file(data: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut gz = GzDecoder::new(&data[..]);
    let mut v = Vec::new();
    gz.read_to_end(&mut v)?;
    Ok(v)
}

pub fn parse_nbt(data: Vec<u8>) {
    todo!()
}
