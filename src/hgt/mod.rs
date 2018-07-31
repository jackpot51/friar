pub use self::cache::HgtCache;
pub use self::file::HgtFile;
pub use self::srtm::HgtSrtm;

mod cache;
mod file;
mod srtm;

#[derive(Clone, Copy)]
pub enum HgtResolution {
    /// One arc-second resolution
    One,
    /// Three arc-second resolution
    Three,
    // Thirty arc-second resolution
    //TODO Thirty,
}

impl HgtResolution {
    /// Return resolution in degrees
    pub fn degrees(&self) -> f64 {
        match *self {
            HgtResolution::One => 1.0 / 3600.0,
            HgtResolution::Three => 3.0 / 3600.0,
            //TODO HgtResolution::Thirty => 30.0 / 3600.0,
        }
    }

    /// Return samples for each axis in the file
    pub fn samples(&self) -> u16 {
        match *self {
            HgtResolution::One => 3601,
            HgtResolution::Three => 1201,
            //TODO HgtResolution::Thirty => 121,
        }
    }
}
