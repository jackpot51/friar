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
use std::{cmp, mem};
use std::collections::HashMap;
use std::fmt::{self, Write};
use std::fs::File;
use std::time::Instant;

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

    fn fill_a<R: Renderer>(&self, r: &mut R, color: Color) {
        use orbclient::renderer::fast_set32;

        let a = self.a;
        let b = self.b;
        let c = self.c;
        let d = color.data | 0xFF000000;

        if a.y == b.y && a.y == c.y {
            return;
        }

        let w = r.width() as i32;
        let h = r.height() as i32;

        /*
        let valid = |p: &Point| {
            p.x >= 0 && p.x < w && p.y >= 0 && p.y < h
        };

        if ! valid(&a) || ! valid(&b) || ! valid(&c) {
            return;
        }
        */

        let dx1 = if b.y - a.y > 0 {
            ((b.x - a.x) as f32)/((b.y - a.y) as f32)
        } else {
            0.0
        };

        let dx2 =  if c.y - a.y > 0 {
            ((c.x - a.x) as f32)/((c.y - a.y) as f32)
        } else {
            0.0
        };

        let dx3 = if c.y - b.y > 0 {
            ((c.x - b.x) as f32)/((c.y - b.y) as f32)
        } else {
            0.0
        };

        //println!("{}, {}, {}", dx1, dx2, dx3);

        let data = r.data_mut();
        let data_ptr = data.as_mut_ptr() as *mut u32;
        let horizline = |x1f: f32, x2f: f32, y: i32| {
            let x1 = cmp::max(x1f.round() as i32, 0);
            let x2 = cmp::min(x2f.round() as i32, w - 1);

            if x1 < x2 && y >= 0 && y < h {
                let offset = y * w + x1;
                let len = x2 + 1 - x1;
                unsafe {
                    fast_set32(data_ptr.offset(offset as isize), d, len as usize);
                }
            }
        };

        let mut sx = a.x as f32;
        let mut ex = a.x as f32;
        let mut sy = a.y;
        if dx1 > dx2 {
            while sy <= b.y {
                horizline(sx, ex, sy);
                sy += 1;
                sx += dx2;
                ex += dx1;
            }
            ex = b.x as f32;
            while sy <= c.y {
                horizline(sx, ex, sy);
                sy += 1;
                sx += dx2;
                ex += dx3;
            }
        } else {
            while sy <= b.y {
                horizline(sx, ex, sy);
                sy += 1;
                sx += dx1;
                ex += dx2;
            }
            sx = b.x as f32;
            sy = b.y;
            while sy <= c.y {
                horizline(sx, ex, sy);
                sy += 1;
                sx += dx3;
                ex += dx2;
            }
        }
    }

    fn fill_b<R: Renderer>(&self, r: &mut R, color: Color) {
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
            let second_half = i>t1.y-t0.y || t1.y==t0.y;
            let segment_height = if second_half { t2.y-t1.y } else { t1.y-t0.y };

            let alpha = (i as f32)/(total_height as f32);
            let beta  = ((i-(if second_half { t1.y-t0.y } else { 0 })) as f32)/(segment_height as f32); // be careful: with above conditions no division by zero here

            let ax = t0.x + (((t2.x-t0.x) as f32)*alpha) as i32;
            let bx = if second_half { t1.x + (((t2.x-t1.x) as f32)*beta) as i32 } else { t0.x + (((t1.x-t0.x) as f32)*beta) as i32 };

            let (minx, maxx) = if ax > bx { (bx, ax) } else { (ax, bx) };
            let x1 = cmp::max(minx, 0);
            let x2 = cmp::min(maxx, w - 1);
            let y = t0.y + i;

            if x1 < x2 && y >= 0 && y < h {
                let offset = y * w + x1;
                let len = x2 + 1 - x1;
                //println!("{}, {}", offset, len);
                unsafe {
                    fast_set32(data_ptr.offset(offset as isize), d, len as usize);
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

fn osm<'r, R: Spheroid>(file: &str, reference: &'r R, bounds: (f64, f64, f64, f64), ground: f64) -> Vec<(Position<'r, R>, Position<'r, R>, Position<'r, R>)> {
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
        match s.replace("'", "").replace(" m", "").parse::<f64>() {
            Ok(height) => if s.ends_with("'") {
                height * 3.28084
            } else {
                height
            },
            Err(err) => {
                println!("Failed to parse height {}: {}", s, err);
                0.0
            }
        }
    };

    let mut triangles = Vec::with_capacity(ways.len());
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

                    if let Some(height) = height_opt {
                        let last_coord_max = last_coordinate.offset(height, 0.0, 90.0);
                        let coord_max = coordinate.offset(height, 0.0, 90.0);

                        triangles.push((
                            last_coord_max.position(),
                            last_coord_min.position(),
                            coord_min.position()
                        ));

                        triangles.push((
                            coord_max.position(),
                            coord_min.position(),
                            last_coord_max.position(),
                        ));
                    }
                }
            }

            last_coordinate_opt = Some(coordinate);
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
    let mut draw_style = 0;
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
                            draw_style += 1;
                            if draw_style >= 3 {
                                draw_style = 0;
                            }
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

            triangles.clear();

            for triangle in triangles_earth.iter() {
                let a_ground = ground_perspective.transform(&triangle.0);
                let a_screen = screen.transform(&a_ground);
                if a_screen.2 < 0.0 {
                    continue;
                }

                let b_ground = ground_perspective.transform(&triangle.1);
                let b_screen = screen.transform(&b_ground);
                if b_screen.2 < 0.0 {
                    continue;
                }

                let c_ground = ground_perspective.transform(&triangle.2);
                let c_screen = screen.transform(&c_ground);
                if c_screen.2 < 0.0 {
                    continue;
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

                let z = (a_screen.2 + b_screen.2 + c_screen.2)/3.0;

                let value = (z.log2() * 32.0).max(32.0).min(255.0) as u8;

                triangles.push((z, Triangle::new(a, b, c), Color::rgb(value, value, value)));
            }

            triangles.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            w.set(Color::rgb(0, 0, 0));

            match draw_style {
                0 => for (_z, triangle, color) in triangles.iter() {
                    triangle.fill_a(&mut w, *color);
                },
                1 => for (_z, triangle, color) in triangles.iter() {
                    triangle.fill_b(&mut w, *color);
                },
                2 => for (_z, triangle, color) in triangles.iter() {
                    triangle.draw(&mut w, *color);
                },
                _ => (),
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
                "Triangles: {}",
                triangles.len()
            );
            y += 16;

            let _ = write!(
                WindowWriter::new(&mut w, 0, y, Color::rgb(0xFF, 0xFF, 0xFF)),
                "Style: {}",
                draw_style
            );
            y += 16;

            let _ = write!(
                WindowWriter::new(&mut w, 0, y, Color::rgb(0xFF, 0xFF, 0xFF)),
                "FPS: {}",
                1.0/time
            );

            w.sync();
        }
    }
}
