#![feature(euclidean_division)]

extern crate friar;
extern crate orbclient;
extern crate osmpbfreader;

use friar::coordinate::Coordinate;
use friar::earth::Earth;
use friar::position::Position;
use friar::reference::Reference;
use friar::spheroid::Spheroid;
use orbclient::{Color, EventOption, Renderer, Window, WindowFlag};
use osmpbfreader::{OsmPbfReader, OsmObj, Node, NodeId, Way, WayId};
use std::collections::HashMap;
use std::fmt::{self, Write};
use std::fs::File;
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

fn paths<'r, R: Spheroid>(file: &str, reference: &'r R, bounds: (f64, f64, f64, f64), ground: f64) -> Vec<(Position<'r, R>, Position<'r, R>)> {
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();
    let mut ways: HashMap<WayId, Way> = HashMap::new();

    for obj_res in OsmPbfReader::new(File::open(file).unwrap()).iter() {
        match obj_res.unwrap() {
            OsmObj::Node(node) => {
                nodes.insert(node.id, node);
            },
            OsmObj::Way(way) => {
                ways.insert(way.id, way);
            },
            _ => ()
        }
    }

    let in_bounds = |coordinate: &Coordinate<'r, R>| -> bool {
        coordinate.latitude > bounds.0 &&
        coordinate.latitude < bounds.2 &&
        coordinate.longitude > bounds.1 &&
        coordinate.longitude < bounds.3
    };

    let parse_height = |s: &String| -> f64 {
        match s.replace(" m", "").parse::<f64>() {
            Ok(height) => {
                height
            },
            Err(err) => {
                println!("Failed to parse height {}: {}", s, err);
                0.0
            }
        }
    };

    let mut paths = Vec::with_capacity(ways.len());
    for (_id, way) in ways.iter() {
        // println!("{:?}", way);

        let min_height = way.tags.get("min_height")
            .map(parse_height)
            .unwrap_or(0.0);

        let height_opt = way.tags.get("height")
            .or(way.tags.get("building:height"))
            .map(parse_height);

        let mut last_coordinate_opt = None;
        for node_id in way.nodes.iter() {
            let node = &nodes[node_id];
            // println!("  {:?}", node);

            let coordinate = reference.coordinate(node.lat(), node.lon(), ground);

            if let Some(last_coordinate) = last_coordinate_opt.take() {
                if in_bounds(&coordinate) && in_bounds(&last_coordinate) {
                    let last_coord_min = last_coordinate.offset(min_height, 0.0, 90.0);
                    let coord_min = coordinate.offset(min_height, 0.0, 90.0);

                    paths.push((
                        last_coord_min.position(),
                        coord_min.position()
                    ));

                    if let Some(height) = height_opt {
                        let last_coord_max = last_coordinate.offset(height, 0.0, 90.0);
                        let coord_max = coordinate.offset(height, 0.0, 90.0);

                        paths.push((
                            last_coord_max.position(),
                            coord_max.position()
                        ));
                        paths.push((
                            last_coord_min.position(),
                            last_coord_max.position()
                        ));
                        paths.push((
                            coord_min.position(),
                            coord_max.position()
                        ));
                    }
                }
            }

            last_coordinate_opt = Some(coordinate);
        }
    }

    paths
}

fn main() {
    let mut w = Window::new_flags(-1, -1, 800, 600, "FRIAR", &[WindowFlag::Async, WindowFlag::Resizable]).unwrap();

    let _ = write!(
        WindowWriter::new(
            &mut w,
            0, 0,
            Color::rgb(0xFF, 0xFF, 0xFF)
        ),
        "Loading"
    );

    w.sync();

    let earth = Earth;

    let origin = earth.coordinate(39.739230, -104.987403, 2000.0);
    let km_sw = origin.offset(1000.0, 225.0, 0.0);
    let km_ne = origin.offset(1000.0, 45.0, 0.0);

    let paths = paths(
        "res/planet_-104.99279,39.73659_-104.98198,39.74187.osm.pbf",
        &earth,
        (
            km_sw.latitude, km_sw.longitude,
            km_ne.latitude, km_ne.longitude,
        ),
        1597.0
    );

    let mut viewer = origin.duplicate();
    let mut heading = 0.0;
    let mut pitch = 270.0;
    let mut roll = 0.0;

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

    let mut redraw = true;
    let mut redraw_times = 2;
    let mut lines = Vec::with_capacity(paths.len());

    let mut last_instant = Instant::now();
    loop {
        let instant = Instant::now();
        let duration = instant.duration_since(last_instant);
        last_instant = instant;
        let time = duration.as_secs() as f64 + duration.subsec_nanos() as f64 / 1000000000.0;
        let speed = 250.0 * time;
        let speed_rot = 90.0 * time;

        let mut found_event = true;
        while found_event {
            found_event = false;

            for event in w.events() {
                found_event = true;

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
                    EventOption::Resize(_resize_event) => {
                        redraw = true;
                    },
                    EventOption::Quit(_quit_event) => return,
                    _ => ()
                }
            }
        }

        if move_forward {
            viewer = viewer.offset(speed, heading, 0.0);
            redraw = true;
        }

        if move_aft {
            viewer = viewer.offset(-speed, heading, 0.0);
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

        if redraw {
            if redraw_times > 0 {
                redraw_times -= 1;
            } else {
                redraw = false;
            }

            let viewer_pos = viewer.position();
            let viewer_rot = viewer.rotation();

            let ground_perspective = viewer_pos.perspective(viewer_rot.0, viewer_rot.1, viewer_rot.2);
            let ground_pos = ground_perspective.position(0.0, 0.0, 0.0);

            let perspective = ground_pos.perspective(pitch + 90.0, roll, heading - 90.0);
            let viewport = perspective.viewport(0.0, 0.0, 1.0);

            let w_w = w.width() as i32;
            let w_h = w.height() as i32;
            let screen = viewport.screen(w_w as f64, w_h as f64, 3600.0);

            lines.clear();

            for path in paths.iter() {
                let a_ground = ground_perspective.transform(&path.0);
                let a_screen = screen.transform(&a_ground);
                let b_ground = ground_perspective.transform(&path.1);
                let b_screen = screen.transform(&b_ground);
                lines.push((a_screen, b_screen, (a_screen.2 + b_screen.2)/2.0, Color::rgb(0xFF, 0xFF, 0xFF)));
            }

            lines.sort_unstable_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

            w.set(Color::rgb(0, 0, 0));

            for line in lines.iter() {
                if line.2 > 0.0 {
                    let a = line.0;
                    let ax = a.0.round() as i32;
                    let ay = a.1.round() as i32;

                    let b = line.1;
                    let bx = b.0.round() as i32;
                    let by = b.1.round() as i32;

                    if
                        ax >= 0 &&
                        ax < w_w &&
                        ay >= 0 &&
                        ay < w_h &&
                        bx >= 0 &&
                        bx < w_w &&
                        by >= 0 &&
                        by < w_h
                    {
                        w.wu_line(ax, ay, bx, by, line.3);
                    }
                }
            }

            let center = (w_w/2 as i32, w_h/2 as i32);
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
    }
}
