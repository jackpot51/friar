extern crate friar;
extern crate orbclient;

use friar::earth::Earth;
use friar::reference::Reference;
use friar::spheroid::Spheroid;
use orbclient::{Color, EventOption, Renderer, Window};

fn main() {
    let mut w = Window::new(-1, -1, 800, 600, "FRIAR").unwrap();

    let earth = Earth;

    let red = earth.coordinate(39.73922277, -104.9888542, 1597.0);
    let green = earth.coordinate(39.73923927, -104.98668697, 1600.0);
    let blue = earth.coordinate(39.73926402, -104.9847987, 1608.0);

    let mut spheres = vec![
        (red.position(), Color::rgb(0xFF, 0x00, 0x00), "red".to_string()),
        (green.position(), Color::rgb(0x00, 0xFF, 0x00), "green".to_string()),
        (blue.position(), Color::rgb(0x00, 0x00, 0xFF), "blue".to_string()),
    ];

    let origin = earth.coordinate(39.73924752, -104.99111798, 1597.0);
    let mut viewer = origin.duplicate();

    let mut redraw = true;
    let mut rx = 90.0;
    let mut ry = 0.0;
    let mut rz = 0.0;
    let mut circles = Vec::with_capacity(spheres.len());
    loop {
        if redraw {
            let viewer_pos = viewer.position();
            let viewer_rot = viewer.rotation(rx, ry, rz);
            let perspective = viewer_pos.perspective(viewer_rot.0, viewer_rot.1, viewer_rot.2);
            let viewport = perspective.viewport(0.0, 0.0, 1.0);
            let screen = viewport.screen(w.width() as f64, w.height() as f64, 4800.0);

            println!("position: {}, {}, {}", viewer.latitude, viewer.longitude, viewer.elevation);
            println!("position ECEF: {}, {}, {}", viewer_pos.x, viewer_pos.y, viewer_pos.z);
            println!("rotation: {}, {}, {}", rx, ry, rz);
            println!("rotation ECEF: {}, {}, {}", viewer_rot.0, viewer_rot.1, viewer_rot.2);

            circles.clear();

            for sphere in spheres.iter() {
                let (px, py, pz) = screen.transform(&sphere.0);
                println!("{}: {}, {}, {}", sphere.2, px, py, pz);
                circles.push((px, py, pz, sphere.1));
            }

            circles.sort_unstable_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

            w.set(Color::rgb(0, 0, 0));

            for circle in circles.iter() {
                if circle.2 > 0.0 {
                    w.circle(circle.0 as i32, circle.1 as i32, -circle.2 as i32, circle.3);
                }
            }

            w.sync();

            redraw = false;
        }

        for event in w.events() {
            match event.to_option() {
                EventOption::Key(key_event) => match key_event.scancode {
                    orbclient::K_W if key_event.pressed => {
                        viewer = viewer.offset(1.0, rx, 0.0);
                        redraw = true;
                    },
                    orbclient::K_S if key_event.pressed => {
                        viewer = viewer.offset(1.0, rx + 180.0, 0.0);
                        redraw = true;
                    },
                    orbclient::K_A if key_event.pressed => {
                        viewer = viewer.offset(1.0, rx + 270.0, 0.0);
                        redraw = true;
                    },
                    orbclient::K_D if key_event.pressed => {
                        viewer = viewer.offset(1.0, rx + 90.0, 0.0);
                        redraw = true;
                    },
                    orbclient::K_Q if key_event.pressed => {
                        viewer = viewer.offset(1.0, 0.0, 90.0);
                        redraw = true;
                    },
                    orbclient::K_E if key_event.pressed => {
                        viewer = viewer.offset(1.0, 0.0, 270.0);
                        redraw = true;
                    },

                    orbclient::K_J if key_event.pressed => {
                        rz += 1.0;
                        redraw = true;
                    },
                    orbclient::K_L if key_event.pressed => {
                        rz -= 1.0;
                        redraw = true;
                    },
                    orbclient::K_I if key_event.pressed => {
                        rx -= 1.0;
                        redraw = true;
                    },
                    orbclient::K_K if key_event.pressed => {
                        rx += 1.0;
                        redraw = true;
                    },
                    orbclient::K_U if key_event.pressed => {
                        ry += 1.0;
                        redraw = true;
                    },
                    orbclient::K_O if key_event.pressed => {
                        ry -= 1.0;
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
