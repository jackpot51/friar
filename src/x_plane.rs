use plain::{self, Plain};
use std::io;
use std::net::UdpSocket;

#[derive(Clone, Copy, Debug, Default)]
#[repr(packed)]
pub struct XPlanePosition {
    pub signature: [u8; 5],
    pub longitude: f64,
    pub latitude: f64,
    pub elevation: f64,
    pub agl: f32,
    pub pitch: f32,
    pub heading: f32,
    pub roll: f32,
    pub speed_east: f32,
    pub speed_up: f32,
    pub speed_south: f32,
    pub roll_rate: f32,
    pub pitch_rate: f32,
    pub yaw_rate: f32,
}

unsafe impl Plain for XPlanePosition {}

pub struct XPlane {
    socket: UdpSocket,
}

impl XPlane {
    pub fn new(remote: &str, rate: u8) -> io::Result<Self> {
        let socket = UdpSocket::bind(("0.0.0.0", 0))?;

        let request = format!("RPOS\0{}\0", rate).into_bytes();
        let sent = socket.send_to(&request, (remote, 49000))?;
        if sent != request.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("sent {} bytes instead of {}", sent, request.len())
            ));
        }

        socket.set_nonblocking(true)?;

        socket.connect((remote, 49001))?;

        Ok(Self {
            socket: socket
        })
    }

    pub fn position(&mut self) -> io::Result<Option<XPlanePosition>> {
        let mut rpos = XPlanePosition::default();

        {
            let response = unsafe { plain::as_mut_bytes(&mut rpos) };
            let received = match self.socket.recv(response) {
                Ok(ok) => ok,
                Err(err) => if err.kind() == io::ErrorKind::WouldBlock {
                    return Ok(None);
                } else {
                    return Err(err);
                }
            };

            if received != response.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("received {} bytes instead of {}", received, response.len())
                ));
            }
        }

        let signature = b"RPOS4";
        if &rpos.signature != signature {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("received signature {:?} instead of {:?}", rpos.signature, signature)
            ));
        }

        Ok(Some(rpos))
    }
}
