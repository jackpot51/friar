#[derive(Debug)]
pub struct Gdl90Heartbeat {
    pub status: [u8; 2],
    pub timestamp: u16,
    pub counts: u16
}

impl Gdl90Heartbeat {
    pub fn new(data: &[u8]) -> Option<Self> {
        if data.len() != 6 {
            return None;
        }

        Some(Self {
            status: [data[0], data[1]],
            timestamp: (data[1] as u16) | ((data[2] as u16) << 8),
            counts: (data[3] as u16) | ((data[4] as u16) << 8)
        })
    }
}
