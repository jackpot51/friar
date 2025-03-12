use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::hgt::{HgtFile, HgtResolution, HgtSrtm};

pub struct HgtCache {
    path: PathBuf,
}

impl HgtCache {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_owned()
        }
    }

    pub fn get(&self, latitude: f64, longitude: f64, resolution: HgtResolution) -> io::Result<HgtFile> {
        let name = format!(
            "{}{:02}{}{:03}",
            if latitude < 0.0 {
                "S"
            } else {
                "N"
            },
            latitude.abs() as u32,
            if longitude < 0.0 {
                "W"
            } else {
                "E"
            },
            longitude.abs() as u32
        );

        let root = match resolution {
            HgtResolution::One => "SRTM1",
            HgtResolution::Three => "SRTM3",
        };

        let path = {
            let mut path = self.path.clone();
            path.push(root);
            path.push(&format!("{}.hgt", name));
            path
        };

        if path.exists() {
            return HgtFile::from_path(latitude, longitude, resolution, path);
        }

        let hgt_file = HgtSrtm::get(latitude, longitude, resolution)?;

        {
            let mut path = self.path.clone();
            path.push(root);
            fs::create_dir_all(&path)?;
            path.push(&format!("{}.hgt", name));
            fs::File::create(&path)?.write_all(&hgt_file.data)?;
        }

        Ok(hgt_file)
    }
}
