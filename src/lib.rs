#![feature(euclidean_division)]

extern crate csv;
extern crate osmpbfreader;
extern crate plain;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate zip;

use std::io;

pub mod coordinate;
pub mod earth;
pub mod gdl90;
pub mod hgt;
pub mod osm;
pub mod ourairports;
pub mod perspective;
pub mod position;
pub mod reference;
pub mod screen;
pub mod spheroid;
pub mod vector;
pub mod viewport;
pub mod unit;
pub mod x_plane;

fn reqwest_err(err: reqwest::Error) -> io::Error {
    io::Error::new(
        io::ErrorKind::Other,
        err
    )
}

fn zip_err(err: zip::result::ZipError) -> io::Error {
    match err {
        zip::result::ZipError::Io(io_err) => io_err,
        _ => io::Error::new(
            io::ErrorKind::Other,
            err
        )
    }
}
