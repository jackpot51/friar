use reqwest;
use std::io::{self, Cursor, Read};
use zip;

use {reqwest_err, zip_err};
use hgt::{HgtFile, HgtResolution};

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

pub struct HgtSrtm;

impl HgtSrtm {
    pub fn get(latitude: f64, longitude: f64, resolution: HgtResolution) -> io::Result<HgtFile> {
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
            HgtResolution::One => ("SRTM1", &SRTM1_DIRS),
            HgtResolution::Three => ("SRTM3", &SRTM3_DIRS),
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

        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("HgtSrtm: failed to find {} in {}", name, root)
        ))
    }
}
