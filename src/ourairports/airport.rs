use reqwest;
use std::io;

use crate::reqwest_err;

#[derive(Debug, Deserialize, Serialize)]
pub struct Airport {
    pub id: u64,
    pub ident: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    pub latitude_deg: Option<f64>,
    pub longitude_deg: Option<f64>,
    pub elevation_ft: Option<f64>,
    pub continent: String,
    pub iso_country: String,
    pub iso_region: String,
    pub municipality: String,
    pub scheduled_service: String,
    pub gps_code: String,
    pub iata_code: String,
    pub local_code: String,
    pub home_link: String,
    pub wikipedia_link: String,
    pub keywords: String,
}

impl Airport {
    pub fn all() -> io::Result<Vec<Self>> {
        let response = reqwest::get("https://davidmegginson.github.io/ourairports-data/airports.csv")
            .map_err(reqwest_err)?;

        let mut entries = Vec::new();

        let csv_reader = csv::ReaderBuilder::new()
            .flexible(true)
            .from_reader(response);
        for entry_res in csv_reader.into_deserialize() {
            entries.push(entry_res?);
        }

        Ok(entries)
    }
}
