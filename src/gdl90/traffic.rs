use std::str;

#[derive(Debug)]
pub enum Gdl90TrafficAddress {
    AdsBIcao(u32),
    AdsBSelf(u32),
    TisBIcao(u32),
    TisBTrack(u32),
    Surface(u32),
    GroundStation(u32),
    Other(u8, u32)
}

#[derive(Debug)]
pub struct Gdl90Traffic {
    pub status: u8,
    pub kind: u8,
    pub address: u32,
    pub latitude: u32,
    pub longitude: u32,
    pub altitude: u16,
    pub misc: u8,
    pub integrity: u8,
    pub accuracy: u8,
    pub h_velocity: u16,
    pub v_velocity: u16,
    pub heading: u8,
    pub category: u8,
    pub callsign: [u8; 8],
    pub priority: u8,
    pub spare: u8,
}

impl Gdl90Traffic {
    pub fn new(data: &[u8]) -> Option<Self> {
        if data.len() != 27 {
            return None;
        }

        Some(Self {
            status: data[0] >> 4,
            kind: data[0] & 0xF,
            address: ((data[1] as u32) << 16) | ((data[2] as u32) << 8) | (data[3] as u32),
            latitude: ((data[4] as u32) << 16) | ((data[5] as u32) << 8) | (data[6] as u32),
            longitude: ((data[7] as u32) << 16) | ((data[8] as u32) << 8) | (data[9] as u32),
            altitude: ((data[10] as u16) << 4) | ((data[11] >> 4) as u16),
            misc: data[11] & 0xF,
            integrity: (data[12] >> 4) & 0xF,
            accuracy: data[12] & 0xF,
            h_velocity: ((data[13] as u16) << 8) | ((data[14] >> 4) as u16),
            v_velocity: (((data[14] & 0xF) as u16) << 8) | (data[15] as u16),
            heading: data[16],
            category: data[17],
            callsign: [
                data[18],
                data[19],
                data[20],
                data[21],
                data[22],
                data[23],
                data[24],
                data[25],
            ],
            priority: data[26] >> 4,
            spare: data[26] & 0xF
        })
    }

    pub fn id(&self) -> u32 {
        ((self.kind as u32) << 24) | self.address
    }

    pub fn address(&self) -> Gdl90TrafficAddress {
        match self.kind {
            0 => Gdl90TrafficAddress::AdsBIcao(self.address),
            1 => Gdl90TrafficAddress::AdsBSelf(self.address),
            2 => Gdl90TrafficAddress::TisBIcao(self.address),
            3 => Gdl90TrafficAddress::TisBTrack(self.address),
            4 => Gdl90TrafficAddress::Surface(self.address),
            5 => Gdl90TrafficAddress::GroundStation(self.address),
            other => Gdl90TrafficAddress::Other(other, self.address)
        }
    }

    pub fn callsign(&self) -> &str {
        let mut i = 0;
        while i < self.callsign.len() {
            if self.callsign[i] == 0 {
                break;
            }
            i += 1;
        }

        unsafe { str::from_utf8_unchecked(&self.callsign[..i + 1]) }
    }

    pub fn latitude(&self) -> f64 {
        let latitude = if self.latitude & 0x800000 == 0 {
            self.latitude as i32
        } else {
            (self.latitude | 0xFF800000) as i32
        };
        (latitude as f64) * 180.0 / 8388608.0
    }

    pub fn longitude(&self) -> f64 {
        let longitude = if self.longitude & 0x800000 == 0 {
            self.longitude as i32
        } else {
            (self.longitude | 0xFF800000) as i32
        };
        (longitude as f64) * 180.0 / 8388608.0
    }

    pub fn altitude(&self) -> f64 {
        (self.altitude as f64) * 25.0 - 1000.0
    }

    pub fn heading(&self) -> f64 {
        (self.heading as f64) * 360.0 / 256.0
    }
}
