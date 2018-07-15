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

    let viewer = earth.coordinate(39.73924752, -104.99111798, 1597.0);
    let viewer_pos = viewer.position();

    let mut redraw = true;
    let mut rx = 90.0;
    let mut ry = 0.0;
    let mut rz = 75.0;
    loop {
        if redraw {
            println!("rotation: {}, {}, {}", rx, ry, rz);

            w.set(Color::rgb(0, 0, 0));

            {
                let perspective = viewer_pos.perspective(rx, ry, rz);
                let viewport = perspective.viewport(0.0, 0.0, 1.0);

                {
                    let name = "red";
                    let color = Color::rgb(0xFF, 0x00, 0x00);
                    let (x, y, z) = viewport.transform(&red_pos);
                    let (px, py, pz) = ((x + 0.5) * 800.0, (y + 0.5) * 600.0, z * 4800.0);
                    println!("{}: {}, {}, {} => {}, {}, {}", name, x, y, z, px, py, pz);
                    w.circle(px as i32, py as i32, -pz as i32, color);
                }

                {
                    let name = "green";
                    let color = Color::rgb(0x00, 0xFF, 0x00);
                    let (x, y, z) = viewport.transform(&green_pos);
                    let (px, py, pz) = ((x + 0.5) * 800.0, (y + 0.5) * 600.0, z * 4800.0);
                    println!("{}: {}, {}, {} => {}, {}, {}", name, x, y, z, px, py, pz);
                    w.circle(px as i32, py as i32, -pz as i32, color);
                }

                {
                    let name = "blue";
                    let color = Color::rgb(0x00, 0x00, 0xFF);
                    let (x, y, z) = viewport.transform(&blue_pos);
                    let (px, py, pz) = ((x + 0.5) * 800.0, (y + 0.5) * 600.0, z * 4800.0);
                    println!("{}: {}, {}, {} => {}, {}, {}", name, x, y, z, px, py, pz);
                    w.circle(px as i32, py as i32, -pz as i32, color);
                }
            }

            w.sync();

            redraw = false;
        }

        for event in w.events() {
            match event.to_option() {
                EventOption::Key(key_event) => match key_event.scancode {
                    orbclient::K_W if key_event.pressed => {
                        rz += 1.0;
                        redraw = true;
                    },
                    orbclient::K_S if key_event.pressed => {
                        rz -= 1.0;
                        redraw = true;
                    },
                    orbclient::K_A if key_event.pressed => {
                        rx -= 1.0;
                        redraw = true;
                    },
                    orbclient::K_D if key_event.pressed => {
                        rx += 1.0;
                        redraw = true;
                    },
                    orbclient::K_Q if key_event.pressed => {
                        ry -= 1.0;
                        redraw = true;
                    },
                    orbclient::K_E if key_event.pressed => {
                        ry += 1.0;
                        redraw = true;
                    },
                    _ => (),
                },
                EventOption::Quit(_quit_event) => return,
                _ => ()
            }
        }
    }
}
