use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub enum HgtFileResolution {
    /// One arc-second resolution
    One,
    /// Three arc-second resolution
    Three,
    // Thirty arc-second resolution
    //TODO Thirty,
}

impl HgtFileResolution {
    /// Return resolution in degrees
    pub fn degrees(&self) -> f64 {
        match *self {
            HgtFileResolution::One => 1.0 / 3600.0,
            HgtFileResolution::Three => 3.0 / 3600.0,
            //TODO HgtFileResolution::Thirty => 30.0 / 3600.0,
        }
    }

    /// Return samples for each axis in the file
    pub fn samples(&self) -> u16 {
        match *self {
            HgtFileResolution::One => 3601,
            HgtFileResolution::Three => 1201,
            //TODO HgtFileResolution::Thirty => 121,
        }
    }
}

pub struct HgtFile {
    /// Identifies the southmost latitude
    pub latitude: f64,
    /// Identifies the westmost longitude
    pub longitude: f64,
    /// Identifies the resolution of the file
    pub resolution: HgtFileResolution,
    /// Data loaded from file
    pub data: Box<[u8]>
}

impl HgtFile {
    /// Creates a new HgtFile from a path, origin in latitude and longitude, and resolution in arc-seconds
    pub fn new<P: AsRef<Path>>(path: P, latitude: f64, longitude: f64, resolution: HgtFileResolution) -> io::Result<Self> {
        let mut file = File::open(path.as_ref())?;

        let expected_len = (resolution.samples() as usize).pow(2) * 2;

        let metadata = file.metadata()?;
        let len = metadata.len();
        if len != expected_len as u64 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("HgtFile: {}: size of {} is not equal to {}", path.as_ref().display(), len, expected_len)
            ));
        }

        let mut data = Vec::with_capacity(expected_len);
        let read_len = file.read_to_end(&mut data)?;
        if read_len != expected_len {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("HgtFile: {}: read of {} is not equal to {}", path.as_ref().display(), len, expected_len)
            ));
        }

        Ok(Self {
            latitude,
            longitude,
            resolution,
            data: data.into_boxed_slice()
        })
    }

    pub fn empty(latitude: f64, longitude: f64, resolution: HgtFileResolution) -> Self {
        let len = (resolution.samples() as usize).pow(2) * 2;

        let mut data = Vec::with_capacity(len);

        for _i in 0..len {
            data.push(0);
        }

        Self {
            latitude,
            longitude,
            resolution,
            data: data.into_boxed_slice()
        }
    }

    /// Get the height in meters at a file position
    pub fn get(&self, row: u16, col: u16) -> Option<i16> {
        let samples = self.resolution.samples();
        if row > 0 && row < samples && col > 0 && col < samples {
            let offset = (((samples - row - 1) as usize) * (samples as usize) + ((col - 1) as usize)) * 2;

            let high = self.data[offset];
            let low = self.data[offset + 1];


            let sample = ((high as i16) << 8) + (low as i16);
            if sample == -32768 {
                None
            } else {
                Some(sample)
            }
        } else {
            println!("HgtFile: {}, {} out of bounds of {}", row, col, samples);
            None
        }
    }

    /// Produce row and column from latitude and longitude
    pub fn position(&self, latitude: f64, longitude: f64) -> Option<(u16, u16)> {
        let res = self.resolution.degrees();
        let row = ((latitude - self.latitude) / res).round() as i64;
        let col = ((longitude - self.longitude) / res).round() as i64;

        let samples = self.resolution.samples() as i64;
        if row > 0 && row < samples && col > 0 && col < samples {
            Some((row as u16, col as u16))
        } else {
            None
        }
    }

    /// Produce latitude and longitude from row and column
    pub fn coordinate(&self, row: u16, col: u16) -> Option<(f64, f64)> {
        let samples = self.resolution.samples();
        if row > 0 && row < samples && col > 0 && col < samples {
            let res = self.resolution.degrees();
            let latitude = (row as f64) * res + self.latitude;
            let longitude = (col as f64) * res + self.longitude;

            Some((latitude, longitude))
        } else {
            None
        }
    }
}
