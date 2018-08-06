extern crate friar;

use friar::gdl90::{Gdl90, Gdl90Kind};
use std::thread;
use std::time::Duration;

fn main() {
    let mut gdl90 = Gdl90::new().unwrap();

    loop {
        if let Some(msg) = gdl90.message().unwrap() {
            println!("{:>02X}", msg.id());
            if let Some(kind) = msg.kind() {
                match kind {
                    Gdl90Kind::Heartbeat(heartbeat) => {
                        println!("{:?}", heartbeat);
                    },
                    Gdl90Kind::Ownship(traffic) | Gdl90Kind::Traffic(traffic) => {
                        println!("{:?} ({}): {}, {}, {}, {}",
                            traffic.address(),
                            traffic.callsign(),
                            traffic.latitude(),
                            traffic.longitude(),
                            traffic.altitude(),
                            traffic.heading()
                        );
                    },
                    Gdl90Kind::GeoAltitude(altitude) => {
                        println!("{:?}", altitude);
                    },
                    Gdl90Kind::ForeFlightAhrs(ahrs) => {
                        println!("{:?}", ahrs);
                    }
                }
            }
        } else {
            thread::sleep(Duration::new(0, 1000000000/1000));
        }
    }
}
