use reqwest;
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};
use zip;

use hgt_file::{HgtFile, HgtFileResolution};

static SRTM_URL: &'static str = "https://dds.cr.usgs.gov/srtm/version2_1";

static SRTM1_DIRS: [&'static str; 7] = [
    "Region_01",
    "Region_02",
    "Region_03",
    "Region_04",
    "Region_05",
    "Region_06",
    "Region_07",
];

static SRTM3_DIRS: [&'static str; 6] = [
    "Africa",
    "Australia",
    "Eurasia",
    "Islands",
    "North_America",
    "South_America",
];

pub struct HgtSrtm {
    path: PathBuf,
}

impl HgtSrtm {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_owned()
        }
    }

    pub fn get(&self, latitude: f64, longitude: f64, resolution: HgtFileResolution) -> io::Result<HgtFile> {
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

        let (root, dirs): (&str, &[&str]) = match resolution {
            HgtFileResolution::One => ("SRTM1", &SRTM1_DIRS),
            HgtFileResolution::Three => ("SRTM3", &SRTM3_DIRS),
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

        let reqwest_err = |err| {
            io::Error::new(
                io::ErrorKind::Other,
                err
            )
        };

        let zip_err = |err| {
            match err {
                zip::result::ZipError::Io(io_err) => io_err,
                _ => io::Error::new(
                    io::ErrorKind::Other,
                    err
                )
            }
        };

        for dir in dirs.iter() {
            let url = format!("{}/{}/{}/{}.hgt.zip", SRTM_URL, root, dir, name);
            println!("{}", url);

            let mut response = reqwest::get(&url).map_err(reqwest_err)?;
            let status = response.status();
            println!("  status {}", status);

            if status.is_success() {
                let mut zip_data = Vec::new();
                response.copy_to(&mut zip_data).map_err(reqwest_err)?;
                println!("  received {} bytes", zip_data.len());

                let mut zip = zip::ZipArchive::new(Cursor::new(zip_data)).map_err(zip_err)?;
                let mut zip_file = zip.by_name(&format!("{}.hgt", name)).map_err(zip_err)?;

                let mut data = Vec::new();
                zip_file.read_to_end(&mut data)?;

                println!("  uncompressed {} bytes", data.len());

                return HgtFile::new(latitude, longitude, resolution, data.into_boxed_slice());
            }
        }

        panic!("todo: download a file for {}", name);
    }
}
