extern crate friar;
extern crate orbclient;

use friar::earth::Earth;
use friar::spheroid::Spheroid;
use orbclient::{Color, EventOption, Renderer, Window};

fn main() {
    let mut w = Window::new(-1, -1, 800, 600, "FRIAR").unwrap();

    let earth = Earth;

    let red = earth.coordinate(39.73922277, -104.9888542, 1597.0);
    let red_pos = red.position();

    let green = earth.coordinate(39.73923927, -104.98668697, 1600.0);
    let green_pos = green.position();

    let blue = earth.coordinate(39.73926402, -104.9847987, 1608.0);
    let blue_pos = blue.position();

    let viewer = earth.coordinate(39.73924752, -104.99111798, -1597.0);
    let viewer_pos = viewer.position();

    let perspective = viewer_pos.perspective(0.0, 0.0, 90.0);
    let viewport = perspective.viewport(1.5, 0.5, 1.0);

    {
        let (x, y, z) = viewport.transform(red_pos);
        println!("red: {}, {}, {} => {}, {}, {}", x, y, z, x * 800.0, y * 600.0, z * 4800.0);
        w.circle((x * 800.0) as i32, (y * 600.0) as i32, -(z * 4800.0) as i32, Color::rgb(0xFF, 0x00, 0x00));
    }

    {
        let (x, y, z) = viewport.transform(green_pos);
        println!("green: {}, {}, {} => {}, {}, {}", x, y, z, x * 800.0, y * 600.0, z * 4800.0);
        w.circle((x * 800.0) as i32, (y * 600.0) as i32, -(z * 4800.0) as i32, Color::rgb(0x00, 0xFF, 0x00));
    }

    {
        let (x, y, z) = viewport.transform(blue_pos);
        println!("blue: {}, {}, {} => {}, {}, {}", x, y, z, x * 800.0, y * 600.0, z * 4800.0);
        w.circle((x * 800.0) as i32, (y * 600.0) as i32, -(z * 4800.0) as i32, Color::rgb(0x00, 0x00, 0xFF));
    }

    w.sync();

    loop {
        for event in w.events() {
            match event.to_option() {
                EventOption::Quit(_quit_event) => return,
                _ => ()
            }
        }
    }
}
