#[derive(Debug)]
pub struct Gdl90ForeFlightAhrs {
    pub roll: i16,
    pub pitch: i16,
    pub heading: u16,
    pub indicated_airspeed: u16,
    pub true_airspeed: u16
}

impl Gdl90ForeFlightAhrs {
    pub fn new(data: &[u8]) -> Option<Self> {
        if data.len() != 10 {
            return None;
        }

        Some(Self {
            roll: ((data[0] as i16) << 8) | (data[1] as i16),
            pitch: ((data[2] as i16) << 8) | (data[3] as i16),
            heading: ((data[4] as u16) << 8) | (data[5] as u16),
            indicated_airspeed: ((data[6] as u16) << 8) | (data[7] as u16),
            true_airspeed: ((data[8] as u16) << 8) | (data[9] as u16),
        })
    }

    pub fn roll(&self) -> Option<f64> {
        if self.roll == 0x7FFF {
            None
        } else {
            Some((self.roll as f64) / 10.0)
        }
    }

    pub fn pitch(&self) -> Option<f64> {
        if self.pitch == 0x7FFF {
            None
        } else {
            Some((self.pitch as f64) / 10.0)
        }
    }

    pub fn magnetic_heading(&self) -> Option<f64> {
        if self.heading == 0xFFFF {
            None
        } else if self.heading & (1 << 15) == (1 << 15) {
            Some(((self.heading & 0x7FFF) as f64) / 10.0)
        } else {
            None
        }
    }

    pub fn true_heading(&self) -> Option<f64> {
        if self.heading == 0xFFFF {
            None
        } else if self.heading & (1 << 15) == 0 {
            Some(((self.heading & 0x7FFF) as f64) / 10.0)
        } else {
            None
        }
    }

    pub fn indicated_airspeed(&self) -> Option<f64> {
        if self.indicated_airspeed == 0xFFFF {
            None
        } else {
            Some(self.indicated_airspeed as f64)
        }
    }

    pub fn true_airspeed(&self) -> Option<f64> {
        if self.true_airspeed == 0xFFFF {
            None
        } else {
            Some(self.true_airspeed as f64)
        }
    }
}
