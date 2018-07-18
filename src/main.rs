#![feature(euclidean_division)]

extern crate friar;
extern crate orbclient;
extern crate osmpbfreader;
extern crate polygon2;
extern crate rayon;

use friar::coordinate::Coordinate;
use friar::earth::Earth;
use friar::position::Position;
use friar::reference::Reference;
use friar::spheroid::Spheroid;
use orbclient::{Color, EventOption, Renderer, Window, WindowFlag};
use osmpbfreader::{OsmPbfReader, OsmObj, Node, NodeId, Way, WayId};
use rayon::prelude::*;
use std::{cmp, mem, thread};
use std::collections::HashMap;
use std::fmt::{self, Write};
use std::fs::File;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

struct Triangle {
    a: Point,
    b: Point,
    c: Point,
}

impl Triangle {
    fn new(mut a: Point, mut b: Point, mut c: Point) -> Self {
        if a.y > b.y {
            mem::swap(&mut a, &mut b);
        }
        if a.y > c.y {
            mem::swap(&mut a, &mut c);
        }
        if b.y > c.y {
            mem::swap(&mut b, &mut c);
        }

        Self {
            a,
            b,
            c
        }
    }

    fn draw<R: Renderer>(&self, r: &mut R, color: Color) {
        let a = self.a;
        let b = self.b;
        let c = self.c;

        let w = r.width() as i32;
        let h = r.height() as i32;

        let valid = |p: &Point| {
            p.x >= 0 && p.x < w && p.y >= 0 && p.y < h
        };

        if ! valid(&a) || ! valid(&b) || ! valid(&c) {
            return;
        }

        r.wu_line(a.x, a.y, b.x, b.y, color);
        r.wu_line(a.x, a.y, c.x, c.y, color);
        r.wu_line(b.x, b.y, c.x, c.y, color);
    }

    // Adapted from https://github.com/ssloy/tinyrenderer/wiki/Lesson-2:-Triangle-rasterization-and-back-face-culling
    fn fill<R: Renderer>(&self, r: &mut R, color: Color) {
        use orbclient::renderer::fast_set32;

        let t0 = self.a;
        let t1 = self.b;
        let t2 = self.c;
        let d = color.data | 0xFF000000;

        if t0.y == t1.y && t0.y == t2.y {
            return;
        }

        let w = r.width() as i32;
        let h = r.height() as i32;

        let data = r.data_mut();
        let data_ptr = data.as_mut_ptr() as *mut u32;

        let total_height = t2.y-t0.y;
        let mut i = 0;
        while i<total_height {
            let y = t0.y + i;

            if y >= h {
                break;
            } else if y >= 0 {
                let second_half = i>t1.y-t0.y || t1.y==t0.y;
                let segment_height = if second_half { t2.y-t1.y } else { t1.y-t0.y };

                let alpha = (i as f32)/(total_height as f32);
                let beta  = ((i-(if second_half { t1.y-t0.y } else { 0 })) as f32)/(segment_height as f32); // be careful: with above conditions no division by zero here

                let ax = t0.x + (((t2.x-t0.x) as f32)*alpha) as i32;
                let bx = if second_half { t1.x + (((t2.x-t1.x) as f32)*beta) as i32 } else { t0.x + (((t1.x-t0.x) as f32)*beta) as i32 };

                let (minx, maxx) = if ax > bx { (bx, ax) } else { (ax, bx) };
                let x1 = cmp::max(minx, 0);
                let x2 = cmp::min(maxx, w - 1);

                if x1 < x2 && y >= 0 && y < h {
                    let offset = y * w + x1;
                    let len = x2 + 1 - x1;
                    //println!("{}, {}", offset, len);
                    unsafe {
                        fast_set32(data_ptr.offset(offset as isize), d, len as usize);
                    }
                }
            }

            i += 1;
        }
    }
}

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

fn osm<'r, R: Spheroid>(file: &str, reference: &'r R, bounds: (f64, f64, f64, f64), ground: f64) -> Vec<(Position<'r, R>, Position<'r, R>, Position<'r, R>, (u8, u8, u8))> {
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

    let check_bounds = |coordinate: &Coordinate<'r, R>| -> bool {
        coordinate.latitude > bounds.0 &&
        coordinate.latitude < bounds.2 &&
        coordinate.longitude > bounds.1 &&
        coordinate.longitude < bounds.3
    };

    let parse_height = |s: &String| -> Option<f64> {
        Some(match s.replace("'", "").replace(" m", "").parse::<f64>() {
            Ok(height) => if s.ends_with("'") {
                height * 3.28084
            } else {
                height
            },
            Err(err) => {
                println!("Failed to parse height {}: {}", s, err);
                return None;
            }
        })
    };

    let parse_color = |s: &String| -> Option<u32> {
        Some(match s.as_str() {
            "black" => 0x404040, //0x000000,
            "white" => 0xFFFFFF,
            "gray" | "grey" => 0x808080,
            "silver" => 0xC0C0C0,
            "maroon" => 0x800000,
            "red" =>  0xFF0000,
            "olive" => 0x808000,
            "yellow" => 0xFFFF00,
            "green" => 0x008000,
            "lime" => 0x00FF00,
            "teal" => 0x008080,
            "aqua" | "cyan" => 0x00FFFF,
            "navy" => 0x000080,
            "blue" => 0x0000FF,
            "purple" => 0x800080,
            "fuchsia" | "magenta" => 0xFF00FF,

            "brown" => 0xA52A2A,
            "darkgray" | "darkgrey" | "dark_gray" | "dark_grey" => 0xA9A9A9,
            "saddlebrown" => 0x8B4513,
            "sandybrown" => 0xF4A460,
            "sienna" => 0xA0522D,
            "tan" => 0xD2B48C,
            _ => {
                let s_trim = if s.starts_with('#') {
                    &s[1..]
                } else {
                    s
                };
                match u32::from_str_radix(&s_trim, 16) {
                    Ok(color) => color,
                    Err(err) => {
                        println!("Failed to parse color {}: {}", s, err);
                        return None;
                    }
                }
            }
        })
    };

    let parse_levels = |s: &String| -> Option<f64> {
        Some(match s.parse::<f64>() {
            Ok(levels) => levels * 3.0,
            Err(err) => {
                println!("Failed to parse levels {}: {}", s, err);
                return None;
            }
        })
    };

    let mut triangles = Vec::with_capacity(ways.len());
    for (_id, way) in ways.iter() {
        // println!("{:?}", way);

        let mut in_bounds = false;
        let mut coords = Vec::with_capacity(way.nodes.len());

        for node_id in way.nodes.iter() {
            let node = &nodes[node_id];
            // println!("  {:?}", node);

            let coord = reference.coordinate(node.lat(), node.lon(), ground);

            if check_bounds(&coord) {
                in_bounds = true;
            }

            coords.push(coord);
        }

        if ! in_bounds {
            continue;
        }

        let min_height = way.tags.get("min_height")
            .and_then(parse_height)
            .unwrap_or(0.0);

        let height_opt = way.tags.get("height")
            .or(way.tags.get("building:height"))
            .and_then(parse_height)
            .or(way.tags.get("building:levels")
            .and_then(parse_levels))
            .or(way.tags.get("building")
            .map(|_| 3.0));

        let color = way.tags.get("building:colour")
            .or(way.tags.get("building:color"))
            .and_then(parse_color)
            .unwrap_or(0xFFFFFF);

        let rgb = (
            (color >> 16) as u8,
            (color >> 8) as u8,
            color as u8
        );

        for i in 1..coords.len() {
            let last_coord = &coords[i - 1];
            let coord = &coords[i];

            let (
                last_coord_min,
                coord_min,
                last_coord_max,
                coord_max,
            ) = if let Some(height) = height_opt {
                (
                    last_coord.offset(min_height, 0.0, 90.0),
                    coord.offset(min_height, 0.0, 90.0),
                    last_coord.offset(height, 0.0, 90.0),
                    coord.offset(height, 0.0, 90.0),
                )
            } else {
                let thickness = 0.25;
                let heading = last_coord.heading(&coord);

                (
                    last_coord.offset(thickness, 270.0 + heading, 0.0),
                    coord.offset(thickness, 270.0 + heading, 0.0),
                    last_coord.offset(thickness, 90.0 + heading, 0.0),
                    coord.offset(thickness, 90.0 + heading, 0.0),
                )
            };

            triangles.push((
                last_coord_max.position(),
                last_coord_min.position(),
                coord_min.position(),
                rgb
            ));

            triangles.push((
                coord_max.position(),
                coord_min.position(),
                last_coord_max.position(),
                rgb
            ));
        }

        if let Some(height) = height_opt {
            if coords.len() >= 3 {
                let roof_color_opt = way.tags.get("roof:colour")
                    .or(way.tags.get("roof:color"))
                    .and_then(parse_color);

                if let Some(roof_color) = roof_color_opt {
                    let roof_rgb = (
                        (roof_color >> 16) as u8,
                        (roof_color >> 8) as u8,
                        roof_color as u8
                    );

                    let mut points = Vec::with_capacity(coords.len());

                    let first = &coords[0];
                    for coord in coords.iter() {
                        let d = first.distance(coord);
                        let h = first.heading(coord);
                        let x = d * h.to_radians().cos();
                        let y = d * h.to_radians().sin();
                        points.push([x, y]);
                    }

                    let indexes = polygon2::triangulate(&points);
                    if indexes.len() < points.len() {
                        println!("{:?} => {:?}", points, indexes);
                    }
                    for chunk in indexes.chunks(3) {
                        let a = coords[chunk[0]].offset(height, 0.0, 90.0);
                        let b = coords[chunk[1]].offset(height, 0.0, 90.0);
                        let c = coords[chunk[2]].offset(height, 0.0, 90.0);

                        triangles.push((
                            a.position(),
                            b.position(),
                            c.position(),
                            roof_rgb,
                        ))
                    }
                }
            }
        }
    }

    triangles
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

    println!("Origin: {}", origin);
    println!("SW: {}", km_sw);
    println!("NE: {}", km_ne);
    println!("OSM: {},{},{},{}", km_sw.longitude, km_sw.latitude, km_ne.longitude, km_ne.latitude);

    let triangles_earth = osm(
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
    let mut fill = true;
    let mut triangles = Vec::with_capacity(triangles_earth.len());

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
                            heading = 0.0;
                            pitch = 270.0;
                            roll = 0.0;
                            redraw = true;
                        },

                        orbclient::K_F if key_event.pressed => {
                            fill = !fill;
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
            let screen = viewport.screen(w_w as f64, w_h as f64);

            triangles.clear();
            triangles.par_extend(
                triangles_earth.par_iter().filter_map(|triangle| {
                    let a_ground = ground_perspective.transform(&triangle.0);
                    let a_screen = screen.transform(&a_ground);
                    if a_screen.2 < 0.0 {
                        return None;
                    }

                    let b_ground = ground_perspective.transform(&triangle.1);
                    let b_screen = screen.transform(&b_ground);
                    if b_screen.2 < 0.0 {
                        return None;
                    }

                    let c_ground = ground_perspective.transform(&triangle.2);
                    let c_screen = screen.transform(&c_ground);
                    if c_screen.2 < 0.0 {
                        return None;
                    }

                    let a = Point {
                        x: a_screen.0 as i32,
                        y: a_screen.1 as i32,
                    };

                    let b = Point {
                        x: b_screen.0 as i32,
                        y: b_screen.1 as i32,
                    };

                    let c = Point {
                        x: c_screen.0 as i32,
                        y: c_screen.1 as i32,
                    };

                    let z = a_screen.2.min(b_screen.2).min(c_screen.2);

                    let value = (z.log2() * 0.25).max(0.25).min(1.0);
                    let cr = (((triangle.3).0 as f64) * value) as u8;
                    let cg = (((triangle.3).1 as f64) * value) as u8;
                    let cb = (((triangle.3).2 as f64) * value) as u8;

                    Some((z, Triangle::new(a, b, c), Color::rgb(cr, cg, cb)))
                })
            );

            triangles.par_sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            w.set(Color::rgb(0, 0, 0));

            for (_z, triangle, color) in triangles.iter() {
                if fill {
                    triangle.fill(&mut w, *color);
                } else {
                    triangle.draw(&mut w, *color);
                }
            }

            let center = (w_w/2 as i32, w_h/2 as i32);
            w.line(center.0 - 5, center.1, center.0 + 5, center.1, Color::rgb(0xFF, 0xFF, 0xFF));
            w.line(center.0, center.1 - 5, center.0, center.1 + 5, Color::rgb(0xFF, 0xFF, 0xFF));

            let mut y = 0;

            let _ = write!(
                WindowWriter::new(&mut w, 0, y, Color::rgb(0xFF, 0xFF, 0xFF)),
                "Coord: {}",
                viewer
            );
            y += 16;

            let _ = write!(
                WindowWriter::new(&mut w, 0, y, Color::rgb(0xFF, 0xFF, 0xFF)),
                "Pos: {}",
                viewer_pos
            );
            y += 16;

            let _ = write!(
                WindowWriter::new(&mut w, 0, y, Color::rgb(0xFF, 0xFF, 0xFF)),
                "Rot: {}, {}, {}",
                heading, pitch, roll
            );
            y += 16;

            let _ = write!(
                WindowWriter::new(&mut w, 0, y, Color::rgb(0xFF, 0xFF, 0xFF)),
                "Triangles (In): {}",
                triangles_earth.len()
            );
            y += 16;

            let _ = write!(
                WindowWriter::new(&mut w, 0, y, Color::rgb(0xFF, 0xFF, 0xFF)),
                "Triangles (Out): {}",
                triangles.len()
            );
            y += 16;

            let _ = write!(
                WindowWriter::new(&mut w, 0, y, Color::rgb(0xFF, 0xFF, 0xFF)),
                "FPS: {}",
                1.0/time
            );

            w.sync();
        } else {
            thread::sleep(Duration::new(0, 1000000000/60));
        }
    }
}
