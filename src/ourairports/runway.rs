use reqwest;
use std::io;

use crate::reqwest_err;

#[derive(Debug, Deserialize, Serialize)]
pub struct Runway {
    pub id: u64,
    pub airport_ref: u64,
    pub airport_ident: String,
    pub length_ft: Option<f64>,
    pub width_ft: Option<f64>,
    pub surface: String,
    pub lighted: u64,
    pub closed: u64,
    pub le_ident: String,
    pub le_latitude_deg: Option<f64>,
    pub le_longitude_deg: Option<f64>,
    pub le_elevation_ft: Option<f64>,
    #[serde(rename = "le_heading_degT")]
    pub le_heading_deg: Option<f64>,
    pub le_displaced_threshold_ft: Option<f64>,
    pub he_ident: String,
    pub he_latitude_deg: Option<f64>,
    pub he_longitude_deg: Option<f64>,
    pub he_elevation_ft: Option<f64>,
    #[serde(rename = "he_heading_degT")]
    pub he_heading_deg: Option<f64>,
    pub he_displaced_threshold_ft: Option<f64>,
}

impl Runway {
    pub fn all() -> io::Result<Vec<Self>> {
        let response = reqwest::get("https://davidmegginson.github.io/ourairports-data/runways.csv")
            .map_err(reqwest_err)?;

        let mut entries = Vec::new();

        let mut csv_reader = csv::ReaderBuilder::new()
            .flexible(true)
            .from_reader(response);

        // Pop extra field caused by trailing comma
        {
            let mut headers = csv_reader.headers()?.clone();
            let len = headers.len();
            if len > 0 {
                headers.truncate(len - 1);
            }
            csv_reader.set_headers(headers);
        }

        for entry_res in csv_reader.deserialize() {
            entries.push(entry_res?);
        }

        Ok(entries)
    }
}
