use std::io;
use std::net::UdpSocket;

pub use self::foreflight::Gdl90ForeFlightAhrs;
pub use self::heartbeat::Gdl90Heartbeat;
pub use self::geo_altitude::Gdl90GeoAltitude;
pub use self::traffic::Gdl90Traffic;

mod foreflight;
mod heartbeat;
mod geo_altitude;
mod traffic;

#[derive(Debug)]
pub enum Gdl90Kind {
    Heartbeat(Gdl90Heartbeat),
    Ownship(Gdl90Traffic),
    GeoAltitude(Gdl90GeoAltitude),
    Traffic(Gdl90Traffic),
    ForeFlightAhrs(Gdl90ForeFlightAhrs)
}

pub struct Gdl90Message<'a> {
    msg: &'a [u8],
}

impl<'a> Gdl90Message<'a> {
    pub fn new(msg: &'a [u8]) -> Option<Self> {
        // Too short (flag byte, message id, message data, fcs (2), flag byte)
        if msg.len() < 6 {
            return None;
        }

        // Lacking flag bytes
        if msg[0] != 0x7E || msg[msg.len() - 1] != 0x7E {
            return None;
        }

        let fcs = (msg[msg.len() - 3] as u16) |
                  ((msg[msg.len() - 2] as u16) << 8);
        //TODO: Check fcs

        Some(Self {
            msg: msg
        })
    }

    pub fn id(&self) -> u8 {
        self.msg[1]
    }

    pub fn data(&self) -> &[u8] {
        &self.msg[2..self.msg.len() - 3]
    }

    pub fn kind(&self) -> Option<Gdl90Kind> {
        let data = self.data();
        let kind = match self.id() {
            0 => Gdl90Kind::Heartbeat(Gdl90Heartbeat::new(data)?),
            10 => Gdl90Kind::Ownship(Gdl90Traffic::new(data)?),
            11 => Gdl90Kind::GeoAltitude(Gdl90GeoAltitude::new(data)?),
            20 => Gdl90Kind::Traffic(Gdl90Traffic::new(data)?),
            0x65 => match data.get(0)? {
                0x01 => Gdl90Kind::ForeFlightAhrs(Gdl90ForeFlightAhrs::new(&data[1..])?),
                _ => return None
            },
            _ => return None
        };
        Some(kind)
    }
}

pub struct Gdl90 {
    socket: UdpSocket,
    buf: [u8; 256]
}

impl Gdl90 {
    pub fn new() -> io::Result<Self> {
        let socket = UdpSocket::bind(("0.0.0.0", 4000))?;

        socket.set_nonblocking(true)?;

        Ok(Self {
            socket,
            buf: [0; 256],
        })
    }

    pub fn message<'a>(&'a mut self) -> io::Result<Option<Gdl90Message<'a>>> {
        let (count, src) = match self.socket.recv_from(&mut self.buf) {
            Ok(ok) => ok,
            Err(err) => if err.kind() == io::ErrorKind::WouldBlock {
                return Ok(None);
            } else {
                return Err(err);
            }
        };

        Ok(Gdl90Message::new(&self.buf[..count]))
    }
}
