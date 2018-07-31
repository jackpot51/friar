use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use hgt::HgtResolution;

pub struct HgtFile {
    /// Identifies the southmost latitude
    pub latitude: f64,
    /// Identifies the westmost longitude
    pub longitude: f64,
    /// Identifies the resolution of the file
    pub resolution: HgtResolution,
    /// Data loaded from file
    pub data: Box<[u8]>
}

impl HgtFile {
    pub fn new(latitude: f64, longitude: f64, resolution: HgtResolution, data: Box<[u8]>) -> io::Result<Self> {
        let expected_len = (resolution.samples() as usize).pow(2) * 2;
        if data.len() != expected_len {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("HgtFile: data size of {} is not equal to {}", data.len(), expected_len)
            ));
        }

        Ok(Self {
            latitude,
            longitude,
            resolution,
            data: data
        })
    }

    /// Creates a new HgtFile from a path, origin in latitude and longitude, and resolution in arc-seconds
    pub fn from_path<P: AsRef<Path>>(latitude: f64, longitude: f64, resolution: HgtResolution, path: P) -> io::Result<Self> {
        let data = {
            let mut file = File::open(path.as_ref())?;
            let metadata = file.metadata()?;
            let mut data = Vec::with_capacity(metadata.len() as usize);
            file.read_to_end(&mut data)?;
            data.into_boxed_slice()
        };

        Ok(Self {
            latitude,
            longitude,
            resolution,
            data
        })
    }

    pub fn from_value(latitude: f64, longitude: f64, resolution: HgtResolution, value: i16) -> Self {
        let data = {
            let high = (value >> 8) as u8;
            let low = value as u8;
            let len = (resolution.samples() as usize).pow(2) * 2;
            let mut data = Vec::with_capacity(len);
            for _i in 0..len/2 {
                data.push(high);
                data.push(low);
            }
            data.into_boxed_slice()
        };

        Self {
            latitude,
            longitude,
            resolution,
            data
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
