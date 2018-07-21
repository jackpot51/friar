extern crate friar;

use friar::x_plane::XPlane;
use std::thread;
use std::time::Duration;

fn main() {
    let mut xplane = XPlane::new("127.0.0.1", 1).unwrap();

    loop {
        if let Some(position) = xplane.position().unwrap() {
            println!("{:#?}", position);
        } else {
            thread::sleep(Duration::new(0, 1000000000/1000));
        }
    }
}
