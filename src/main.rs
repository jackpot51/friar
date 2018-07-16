#![feature(euclidean_division)]

extern crate friar;
extern crate orbclient;

use friar::earth::Earth;
use friar::reference::Reference;
use friar::spheroid::Spheroid;
use orbclient::{Color, EventOption, Renderer, Window, WindowFlag};
use std::fmt::{self, Write};
use std::time::Instant;

struct WindowWriter<'a> {
    window: &'a mut Window,
    x: i32,
    y: i32,
    color: Color,
}

impl<'a> WindowWriter<'a> {
    fn new(window: &'a mut Window, x: i32, y: i32, color: Color) -> Self {
        Self {
            window,
            x,
            y,
            color
        }
    }
}

impl<'a> fmt::Write for WindowWriter<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        for c in s.chars() {
            self.window.char(self.x, self.y, c, self.color);
            self.x += 8;
        }

        Ok(())
    }
}

fn main() {
    let mut w = Window::new_flags(-1, -1, 800, 600, "FRIAR", &[WindowFlag::Async]).unwrap();

    let earth = Earth;

    let red = earth.coordinate(39.73922277, -104.9888542, 1597.0);
    let orange = earth.coordinate(39.739949, -104.988843, 1597.0);
    let yellow = earth.coordinate(39.738573, -104.988848 , 1597.0);
    let green = earth.coordinate(39.73923927, -104.98668697, 1600.0);
    let blue = earth.coordinate(39.73926402, -104.9847987, 1608.0);

    let spheres = vec![
        (red.position(), Color::rgb(0xFF, 0x00, 0x00), "red".to_string()),
        (orange.position(), Color::rgb(0xFF, 0x7F, 0x00), "orange".to_string()),
        (yellow.position(), Color::rgb(0xFF, 0xFF, 0x00), "yellow".to_string()),
        (green.position(), Color::rgb(0x00, 0xFF, 0x00), "green".to_string()),
        (blue.position(), Color::rgb(0x00, 0x00, 0xFF), "blue".to_string()),
    ];

    let origin = earth.coordinate(39.73922277, -104.99111798, 1599.0);
    let mut viewer = origin.duplicate();

    let mut redraw = true;
    let mut first = true;
    let mut heading = viewer.heading(&red);
    let mut pitch = 0.0;
    let mut roll = 0.0;
    let mut circles = Vec::with_capacity(spheres.len());
    let mut move_left = false;
    let mut move_right = false;
    let mut move_up = false;
    let mut move_down = false;
    let mut move_forward = false;
    let mut move_aft = false;
    let mut rotate_left = false;
    let mut rotate_right = false;
    let mut rotate_up = false;
    let mut rotate_down = false;
    let mut roll_left = false;
    let mut roll_right = false;
    let mut last_instant = Instant::now();
    loop {
        let instant = Instant::now();
        let duration = instant.duration_since(last_instant);
        last_instant = instant;
        let time = duration.as_secs() as f64 + duration.subsec_nanos() as f64 / 1000000000.0;
        let speed = 100.0 * time;
        let speed_rot = 90.0 * time;

        if redraw {
            if first {
                first = false;
            } else {
                redraw = false;
            }

            let viewer_pos = viewer.position();
            let viewer_rot = viewer.rotation();
            let ground_perspective = viewer_pos.perspective(viewer_rot.0, viewer_rot.1, viewer_rot.2);
            let ground_pos = ground_perspective.position(0.0, 0.0, 0.0);
            let perspective = ground_pos.perspective(pitch + 90.0, roll, heading - 90.0);
            let viewport = perspective.viewport(0.0, 0.0, 1.0);
            let screen = viewport.screen(w.width() as f64, w.height() as f64, 3600.0);

            circles.clear();

            for sphere in spheres.iter() {
                let sphere_ground = ground_perspective.transform(&sphere.0);
                let (px, py, pz) = screen.transform(&sphere_ground);
                //println!("{}: {} => {} => {}, {}, {}", sphere.2, sphere.0, sphere_ground, px, py, pz);
                circles.push((px, py, pz, sphere.1));
            }

            circles.sort_unstable_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

            w.set(Color::rgb(0, 0, 0));

            for circle in circles.iter() {
                if circle.2 > 0.0 {
                    w.circle(circle.0 as i32, circle.1 as i32, -circle.2 as i32, circle.3);
                }
            }

            let center = (w.width() as i32/2, w.height() as i32/2);
            w.line(center.0 - 5, center.1, center.0 + 5, center.1, Color::rgb(0xFF, 0xFF, 0xFF));
            w.line(center.0, center.1 - 5, center.0, center.1 + 5, Color::rgb(0xFF, 0xFF, 0xFF));

            let _ = write!(
                WindowWriter::new(&mut w, 0, 0, Color::rgb(0xFF, 0xFF, 0xFF)),
                "FPS: {}",
                1.0/time
            );

            let _ = write!(
                WindowWriter::new(&mut w, 0, 16, Color::rgb(0xFF, 0xFF, 0xFF)),
                "Coord: {}",
                viewer
            );

            let _ = write!(
                WindowWriter::new(&mut w, 0, 32, Color::rgb(0xFF, 0xFF, 0xFF)),
                "Pos: {}",
                viewer_pos
            );

            let _ = write!(
                WindowWriter::new(&mut w, 0, 48, Color::rgb(0xFF, 0xFF, 0xFF)),
                "Rot: {}, {}, {}",
                heading, pitch, roll
            );

            w.sync();
        }

        for event in w.events() {
            match event.to_option() {
                EventOption::Key(key_event) => match key_event.scancode {
                    orbclient::K_W => {
                        move_forward = key_event.pressed;
                    },
                    orbclient::K_S => {
                        move_aft = key_event.pressed;
                    },
                    orbclient::K_A => {
                        move_left = key_event.pressed;
                    },
                    orbclient::K_D => {
                        move_right = key_event.pressed;
                    },
                    orbclient::K_Q => {
                        move_up = key_event.pressed;
                    },
                    orbclient::K_E => {
                        move_down = key_event.pressed;
                    },
                    orbclient::K_R if key_event.pressed => {
                        viewer = origin.duplicate();
                        redraw = true;
                    },

                    orbclient::K_J => {
                        rotate_left = key_event.pressed;
                    },
                    orbclient::K_L => {
                        rotate_right = key_event.pressed;
                    },
                    orbclient::K_I => {
                        rotate_down = key_event.pressed;
                    },
                    orbclient::K_K => {
                        rotate_up = key_event.pressed;
                    },
                    orbclient::K_U => {
                        roll_left = key_event.pressed;
                    },
                    orbclient::K_O => {
                        roll_right = key_event.pressed;
                    },
                    orbclient::K_P if key_event.pressed => {
                        heading = 90.0;
                        pitch = 0.0;
                        roll = 0.0;
                        redraw = true;
                    },

                    _ => (),
                },
                EventOption::Quit(_quit_event) => return,
                _ => ()
            }
        }

        if move_forward {
            viewer = viewer.offset(speed, heading, pitch);
            redraw = true;
        }

        if move_aft {
            viewer = viewer.offset(-speed, heading, pitch);
            redraw = true;
        }

        if move_left {
            viewer = viewer.offset(-speed, heading + 90.0, 0.0);
            redraw = true;
        }

        if move_right {
            viewer = viewer.offset(speed, heading + 90.0, 0.0);
            redraw = true;
        }

        if move_up {
            viewer = viewer.offset(speed, 0.0, 90.0);
            redraw = true;
        }

        if move_down {
            viewer = viewer.offset(-speed, 0.0, 90.0);
            redraw = true;
        }

        if rotate_left {
            heading = (heading - speed_rot).mod_euc(360.0);
            redraw = true;
        }

        if rotate_right {
            heading = (heading + speed_rot).mod_euc(360.0);
            redraw = true;
        }

        if rotate_up {
            pitch = (pitch + speed_rot).mod_euc(360.0);
            redraw = true;
        }

        if rotate_down {
            pitch = (pitch - speed_rot).mod_euc(360.0);
            redraw = true;
        }

        if roll_left {
            roll = (roll - speed_rot).mod_euc(360.0);
            redraw = true;
        }

        if roll_right {
            roll = (roll + speed_rot).mod_euc(360.0);
            redraw = true;
        }
    }
}
