#[derive(Debug)]
pub struct Gdl90GeoAltitude {
    pub altitude: i16,
    pub metrics: u16,
}

impl Gdl90GeoAltitude {
    pub fn new(data: &[u8]) -> Option<Self> {
        if data.len() != 4 {
            return None;
        }

        Some(Self {
            altitude: ((data[0] as i16) << 8) | (data[1] as i16),
            metrics: ((data[2] as u16) << 8) | (data[3] as u16)
        })
    }

    pub fn altitude(&self) -> f64 {
        (self.altitude as f64) * 5.0
    }
}
