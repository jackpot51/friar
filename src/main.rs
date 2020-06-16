extern crate dashmap;
extern crate friar;
extern crate orbclient;
extern crate orbfont;
extern crate polygon2;
extern crate rayon;

use dashmap::DashMap;
use friar::coordinate::Coordinate;
use friar::earth::Earth;
use friar::gdl90::{Gdl90, Gdl90Kind};
use friar::hgt::{HgtCache, HgtFile, HgtResolution};
use friar::osm::Osm;
use friar::ourairports;
use friar::position::Position;
use friar::reference::Reference;
use friar::spheroid::Spheroid;
use friar::x_plane::XPlane;
use orbclient::{Color, EventOption, Renderer, Window, WindowFlag};
use orbfont::{Font, Text};
use rayon::prelude::*;
use std::{cmp, mem, thread};
use std::collections::HashMap;
use std::fmt::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Timer {
    name: &'static str,
    print: bool,
    instant: Instant,
}

impl Timer {
    fn new(name: &'static str, print: bool) -> Self {
        let instant = Instant::now();
        Self {
            name,
            print,
            instant,
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if self.print {
            let duration = self.instant.elapsed();
            println!(
                "{}: {}",
                self.name,
                (duration.as_secs() as f64) + (duration.subsec_nanos() as f64)/1000000000.0
            );
        }
    }
}

#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
    z: f32,
    intensity: f32,
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

        assert!(a.y <= b.y && b.y <= c.y);

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
    fn fill<R: Renderer>(&self, r: &mut R, z_buffer: &mut [f32], color: Color) {
        let w = r.width() as i32;
        let h = r.height() as i32;
        let data = r.data_mut();

        let a = self.a;
        let b = self.b;
        let c = self.c;

        // if a.y == b.y && a.y == c.y {
        //     return;
        // }

        let y1 = cmp::max(a.y, 0);
        let y2 = cmp::min(c.y, h - 1);
        if y1 < y2 {
            let aymcy = a.y - c.y;
            let bymay = b.y - a.y;
            let bymcy = b.y - c.y;
            let cymay = c.y - a.y;
            let cymby = c.y - b.y;
            let axmcx = a.x - c.x;
            let bxmax = b.x - a.x;
            let cxmax = c.x - a.x;
            let cxmbx = c.x - b.x;

            //TODO: Reduce operations in loop
            for y in y1..y2 + 1 {
                let i = y - a.y;

                let second_half = i>bymay || b.y==a.y;
                let segment_height = if second_half { cymby } else { bymay };

                let alpha = (i as f32)/(cymay as f32);
                let beta  = ((i-(if second_half { bymay } else { 0 })) as f32)/(segment_height as f32); // be careful: with above conditions no division by zero here

                let alphax = a.x + ((cxmax as f32)*alpha) as i32;
                let betax = if second_half { b.x + ((cxmbx as f32)*beta) as i32 } else { a.x + ((bxmax as f32)*beta) as i32 };

                let (minx, maxx) = if alphax > betax { (betax, alphax) } else { (alphax, betax) };
                let x1 = cmp::max(minx, 0);
                let x2 = cmp::min(maxx, w - 1);

                if x1 < x2 {
                    let ymcy = y - c.y;

                    let offset_y = (y * w) as usize;

                    for x in x1..x2 + 1 {
                        let xmcx = x - c.x;

                        let wa = (bymcy * xmcx + cxmbx * ymcy) as f32
                            /
                            (bymcy * axmcx + cxmbx * aymcy) as f32;

                        let wb = (cymay * xmcx + axmcx * ymcy) as f32
                            /
                            (bymcy * axmcx + cxmbx * aymcy) as f32;

                        let wc = 1.0 - wa - wb;

                        let weight = |va: f32, vb: f32, vc: f32| -> f32{
                            (wa * va + wb * vb + wc * vc)
                            /
                            (wa + wb + wc)
                        };

                        let z = weight(a.z, b.z, c.z);

                        let offset = offset_y + x as usize;
                        if z_buffer[offset] < z {
                            let intensity = weight(a.intensity, b.intensity, c.intensity);
                            let pixel = Color::rgb(
                                // The following is to debug z-buffer
                                // (z * 16384.0).min(255.0) as u8,
                                // (z * 16384.0).min(255.0) as u8,
                                // (z * 16384.0).min(255.0) as u8
                                (color.r() as f32 * intensity).min(255.0) as u8,
                                (color.g() as f32 * intensity).min(255.0) as u8,
                                (color.b() as f32 * intensity).min(255.0) as u8
                            );

                            data[offset] = pixel;
                            z_buffer[offset] = z;
                        }
                    }
                }
            }
        }
    }

    fn par_fill(&self, w: i32, h: i32, data_ptr: *mut Color, z_ptr: *mut f32, color: Color, row_mutexes: &[Mutex<()>]) {
        let a = self.a;
        let b = self.b;
        let c = self.c;

        // if a.y == b.y && a.y == c.y {
        //     return;
        // }

        let y1 = cmp::max(a.y, 0);
        let y2 = cmp::min(c.y, h - 1);
        if y1 < y2 {
            let aymcy = a.y - c.y;
            let bymay = b.y - a.y;
            let bymcy = b.y - c.y;
            let cymay = c.y - a.y;
            let cymby = c.y - b.y;
            let axmcx = a.x - c.x;
            let bxmax = b.x - a.x;
            let cxmax = c.x - a.x;
            let cxmbx = c.x - b.x;

            //TODO: Reduce operations in loop
            for y in y1..y2 + 1 {
                let i = y - a.y;

                let second_half = i>bymay || b.y==a.y;
                let segment_height = if second_half { cymby } else { bymay };

                let alpha = (i as f32)/(cymay as f32);
                let beta  = ((i-(if second_half { bymay } else { 0 })) as f32)/(segment_height as f32); // be careful: with above conditions no division by zero here

                let alphax = a.x + ((cxmax as f32)*alpha) as i32;
                let betax = if second_half { b.x + ((cxmbx as f32)*beta) as i32 } else { a.x + ((bxmax as f32)*beta) as i32 };

                let (minx, maxx) = if alphax > betax { (betax, alphax) } else { (alphax, betax) };
                let x1 = cmp::max(minx, 0);
                let x2 = cmp::min(maxx, w - 1);

                if x1 < x2 {
                    let ymcy = y - c.y;

                    let offset_y = (y * w) as usize;
                    let _row_lock = row_mutexes[y as usize].lock();

                    for x in x1..x2 + 1 {
                        let xmcx = x - c.x;

                        let wa = (bymcy * xmcx + cxmbx * ymcy) as f32
                            /
                            (bymcy * axmcx + cxmbx * aymcy) as f32;

                        let wb = (cymay * xmcx + axmcx * ymcy) as f32
                            /
                            (bymcy * axmcx + cxmbx * aymcy) as f32;

                        let wc = 1.0 - wa - wb;

                        let weight = |va: f32, vb: f32, vc: f32| -> f32{
                            (wa * va + wb * vb + wc * vc)
                            /
                            (wa + wb + wc)
                        };

                        let z = weight(a.z, b.z, c.z);

                        let offset = offset_y + x as usize;
                        if unsafe { *z_ptr.add(offset) < z } {
                            let intensity = weight(a.intensity, b.intensity, c.intensity);
                            let pixel = Color::rgb(
                                // The following is to debug z-buffer
                                // (z * 16384.0).min(255.0) as u8,
                                // (z * 16384.0).min(255.0) as u8,
                                // (z * 16384.0).min(255.0) as u8
                                (color.r() as f32 * intensity).min(255.0) as u8,
                                (color.g() as f32 * intensity).min(255.0) as u8,
                                (color.b() as f32 * intensity).min(255.0) as u8
                            );

                            unsafe {
                                *data_ptr.add(offset) = pixel;
                                *z_ptr.add(offset) = z;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn line_f64(window: &mut Window, ax: f64, ay: f64, bx: f64, by: f64, color: Color) {
    //TODO: Clean up
    //TODO: Handle overflows (when converting to i32, for example)
    let r = color.r();
    let g = color.g();
    let b = color.b();
    let alpha = color.a() as f64;

    let w_w = window.width() as i32;
    let w_h = window.height() as i32;

    let dx = bx - ax;
    let dy = by - ay;

    if dy.abs() > dx.abs() {
        let slope = dx / dy;

        let start_y = cmp::max(ay.min(by).ceil() as i32, 0);
        let end_y = cmp::min(ay.max(by).floor() as i32, w_h - 1);

        //TODO: Do endpoints, start_y - 1, end_y + 1

        let mut y = start_y;
        while y <= end_y {
            let x = ((y as f64) - ay) * slope + ax;
            let center = x.round() as i32;
            let low = center - 1;
            let high = center + 1;

            if low >= 0 && high < w_w {
                let low_dist = (x - low as f64).abs();
                let center_dist = (x - center as f64).abs();
                let high_dist = (x - high as f64).abs();
                let total_dist = low_dist + center_dist + high_dist;

                let low_weight = 1.0 - low_dist / total_dist;
                window.pixel(low, y, Color::rgba(
                    r,
                    g,
                    b,
                    (alpha * low_weight) as u8
                ));

                let center_weight = 1.0 - center_dist / total_dist;
                window.pixel(center, y, Color::rgba(
                    r,
                    g,
                    b,
                    (alpha * center_weight) as u8
                ));

                let high_weight = 1.0 - high_dist / total_dist;
                window.pixel(high, y, Color::rgba(
                    r,
                    g,
                    b,
                    (alpha * high_weight) as u8
                ));
            }

            y += 1;
        }
    } else {
        let slope = dy / dx;

        let start_x = cmp::max(ax.min(bx).ceil() as i32, 0);
        let end_x = cmp::min(ax.max(bx).floor() as i32, w_w - 1);

        //TODO: Do endpoints, start_x - 1, end_x + 1

        let mut x = start_x;
        while x <= end_x {
            let y = ((x as f64) - ax) * slope + ay;
            let center = y.round() as i32;
            let low = center - 1;
            let high = center + 1;

            if low >= 0 && high < w_h {
                let low_dist = (y - low as f64).abs();
                let center_dist = (y - center as f64).abs();
                let high_dist = (y - high as f64).abs();
                let total_dist = low_dist + center_dist + high_dist;

                let low_weight = 1.0 - low_dist / total_dist;
                window.pixel(x, low, Color::rgba(
                    r,
                    g,
                    b,
                    (alpha * low_weight) as u8
                ));

                let center_weight = 1.0 - center_dist / total_dist;
                window.pixel(x, center, Color::rgba(
                    r,
                    g,
                    b,
                    (alpha * center_weight) as u8
                ));

                let high_weight = 1.0 - high_dist / total_dist;
                window.pixel(x, high, Color::rgba(
                    r,
                    g,
                    b,
                    (alpha * high_weight) as u8
                ));
            }

            x += 1;
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

struct FontCache<'a> {
    font: &'a Font,
    height: f32,
    cache: HashMap<String, Text<'a>>
}

impl<'a> FontCache<'a> {
    fn new(font: &'a Font, height: f32) -> Self {
        Self {
            font,
            height,
            cache: HashMap::new()
        }
    }

    fn render(&mut self, string: &str) -> &Text<'a> {
        if ! self.cache.contains_key(string) {
            self.cache.insert(
                string.to_string(),
                self.font.render(string, self.height)
            );
        }

        &self.cache[string]
    }
}

fn osm_way_triangles<'r, R: Spheroid>(osm: &Osm, reference: &'r R, bounds: (f64, f64, f64, f64), ground: f64, triangles: &mut Vec<(Position<'r, R>, Position<'r, R>, Position<'r, R>, (f32, f32, f32), (u8, u8, u8))>) {
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

    for (_id, way) in osm.ways.iter() {
        // println!("{:?}", way);

        let mut in_bounds = false;
        let mut coords = Vec::with_capacity(way.nodes.len());

        for node_id in way.nodes.iter() {
            let node = &osm.nodes[node_id];
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
                (1.0, 1.0, 1.0),
                rgb
            ));

            triangles.push((
                coord_max.position(),
                coord_min.position(),
                last_coord_max.position(),
                (1.0, 1.0, 1.0),
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
                        // println!("{:?} => {:?}", points, indexes);
                    }
                    for chunk in indexes.chunks(3) {
                        let a = coords[chunk[0]].offset(height, 0.0, 90.0);
                        let b = coords[chunk[1]].offset(height, 0.0, 90.0);
                        let c = coords[chunk[2]].offset(height, 0.0, 90.0);

                        triangles.push((
                            a.position(),
                            b.position(),
                            c.position(),
                            (1.0, 1.0, 1.0),
                            roof_rgb,
                        ))
                    }
                }
            }
        }
    }
}

struct RunwayEnd<'r, R: Spheroid> {
    airport_ident: String,
    ident: String,
    coord: Coordinate<'r, R>,
    length: f64,
    width: f64,
    heading: f64
}

fn oap_runways<'r, R: Spheroid>(reference: &'r R) -> Vec<RunwayEnd<'r, R>> {
    let mut runways = Vec::new();

    for runway in &ourairports::Runway::all().unwrap() {
        let le = || -> Option<RunwayEnd<R>> {
            Some(RunwayEnd {
                airport_ident: runway.airport_ident.clone(),
                ident: runway.le_ident.clone(),
                coord: Coordinate::new(
                    reference,
                    runway.le_latitude_deg?,
                    runway.le_longitude_deg?,
                    runway.le_elevation_ft? * 0.3048 + 20.0
                ),
                length: runway.length_ft? * 0.3048,
                width: runway.width_ft? * 0.3048,
                heading: runway.le_heading_deg?
            })
        };

        let he = || -> Option<RunwayEnd<R>> {
            Some(RunwayEnd {
                airport_ident: runway.airport_ident.clone(),
                ident: runway.he_ident.clone(),
                coord: Coordinate::new(
                    reference,
                    runway.he_latitude_deg?,
                    runway.he_longitude_deg?,
                    runway.he_elevation_ft? * 0.3048 + 20.0
                ),
                length: runway.length_ft? * 0.3048,
                width: runway.width_ft? * 0.3048,
                heading: runway.he_heading_deg?
            })
        };

        if let Some(e) = le() {
            runways.push(e);
        }

        if let Some(e) = he() {
            runways.push(e);
        }
    }

    runways
}

fn oap_runway_triangles<'r, R: Spheroid>(runways: &[RunwayEnd<'r, R>], bounds: (f64, f64, f64, f64), triangles: &mut Vec<(Position<'r, R>, Position<'r, R>, Position<'r, R>, (f32, f32, f32), (u8, u8, u8))>) {
    let check_bounds = |coordinate: &Coordinate<'r, R>| -> bool {
        coordinate.latitude > bounds.0 &&
        coordinate.latitude < bounds.2 &&
        coordinate.longitude > bounds.1 &&
        coordinate.longitude < bounds.3
    };

    for e in runways {
        if ! check_bounds(&e.coord) {
            continue;
        }

        let mut length = 0.0;
        let block = e.width/2.0;
        while length < e.length {
            let coord = e.coord.offset(length, e.heading, 0.0);
            length += block;

            let a = coord.offset(block, e.heading - 90.0, 0.0);
            let b = coord.offset(block, e.heading + 90.0, 0.0);
            let c = a.offset(block, e.heading, 0.0);
            let d = b.offset(block, e.heading, 0.0);

            triangles.push((
                a.position(),
                b.position(),
                c.position(),
                (1.0, 1.0, 1.0),
                (0xAA, 0xAA, 0xAA),
            ));

            triangles.push((
                b.position(),
                c.position(),
                d.position(),
                (1.0, 1.0, 1.0),
                (0xAA, 0xAA, 0xAA),
            ));
        }
    }
}

fn hgt_intersect<'r, R: Spheroid>(file: &HgtFile, reference: &'r R, origin: &Coordinate<'r, R>, heading: f64, pitch: f64) -> Option<Coordinate<'r, R>> {
    let mut a = origin.duplicate();
    let mut a_h = file.interpolate(a.latitude, a.longitude)?;
    let mut a_dh = a.elevation - a_h;
    //TODO: Consider parallelizing
    loop {
        let b = a.offset(1.0, heading, pitch);
        let b_h = file.interpolate(b.latitude, b.longitude)?;
        let b_dh = b.elevation - b_h;

        // Upon transition, return b
        // TODO: Find correct intersect point
        if (a_dh.is_sign_negative() && b_dh.is_sign_positive()) ||
           (a_dh.is_sign_positive() && b_dh.is_sign_negative())
        {
            // println!("origin: {}, heading: {}, pitch: {}", origin, heading, pitch);
            // println!(
            //     "a: {}, a_h: {}, a_dh: {}, b: {}, b_h: {}, b_dh: {}",
            //     a, a_h, a_dh,
            //     b, b_h, b_dh
            // );
            return Some(reference.coordinate(
                (a.latitude + b.latitude)/2.0,
                (a.longitude + b.longitude)/2.0,
                (a_h + b_h)/2.0
            ));
        }

        //TODO: Find easy ways to exit loop early

        a = b;
        a_h = b_h;
        a_dh = b_dh;
    }
}

type HgtTriangle<'r, R> = (Position<'r, R>, Position<'r, R>, Position<'r, R>, (f32, f32, f32), (u8, u8, u8));

type HgtTile<'r, R> = (HgtTriangle<'r, R>, HgtTriangle<'r, R>);

struct HgtFileTiles<'r, R: Spheroid + 'r> {
    file: HgtFile,
    tiles_levels: Box<[Box<[Box<[Option<HgtTile<'r, R>>]>]>]>
}

impl<'r, R: Spheroid + Sync> HgtFileTiles<'r, R> {
    fn new(file: HgtFile, reference: &'r R, ground_color: Color, ocean_color: Color, levels: usize) -> Self {
        let samples = file.resolution.samples();

        let rgb = |low: f64, high: f64| -> (u8, u8, u8) {
            let scale = 1.0; //(1.0 - (high - low).log2() * 0.125).max(0.125).min(1.0);
            let color = if low.abs() < 1.0 && high.abs() < 1.0 {
                ocean_color
            } else {
                ground_color
            };
            (
                (color.r() as f64 * scale) as u8,
                (color.g() as f64 * scale) as u8,
                (color.b() as f64 * scale) as u8
            )
        };

        let (center_lat, center_lon) = file.coordinate(samples/2, samples/2).unwrap();
        let center = reference.coordinate(center_lat, center_lon, 0.0);
        let center_pos = center.position();

        let down = center.offset(1.0, 0.0, 270.0);
        let down_pos = down.position();
        let down_vec = center_pos.vector(&down_pos).normalize();

        let intensity = |p1: &Position<'r, R>, p2: &Position<'r, R>, p3: &Position<'r, R>| -> f32 {
            let v12 = p1.vector(&p2);
            let v13 = p1.vector(&p3);

            let dot = v12.cross(&v13).normalize().dot(&down_vec);
            dot.powi(10) as f32
        };

        let intensity_intense = |p1: &Position<'r, R>, p2: &Position<'r, R>, p3: &Position<'r, R>, c4_opt: &Option<Coordinate<'r, R>>, c5_opt: &Option<Coordinate<'r, R>>| -> f32 {
            let mut int = intensity(p1, p2, p3);
            let mut count = 1.0;

            if let Some(c4) = c4_opt {
                let p4 = c4.position();
                int += intensity(p1, p3, &p4);
                count += 1.0;

                if let Some(c5) = c5_opt {
                    let p5 = c5.position();
                    int += intensity(p1, &p4, &p5);
                    count += 1.0;
                }
            }

            if let Some(c5) = c5_opt {
                let p5 = c5.position();
                int += intensity(p1, &p5, p2);
                count += 1.0;
            }

            int/count
        };

        let mut tiles_levels = Vec::with_capacity(levels);
        for level in 0..levels {
            let cell_map = |prev_row: u16, prev_col: u16, row: u16, col: u16| -> Option<HgtTile<'r, R>> {
                let coord = |row, col| -> Option<Coordinate<'r, R>> {
                    let h = file.get(row, col)?;
                    let (lat, lon) = file.coordinate(row, col)?;
                    Some(reference.coordinate(lat, lon, h as f64))
                };

                //   au bu
                // al a b br
                // cl c d dr
                //   cd dd

                let a = coord(prev_row, prev_col)?;
                let b = coord(prev_row, col)?;
                let c = coord(row, prev_col)?;
                let d = coord(row, col)?;

                let (au, bu) = if prev_row > 1 {
                    (coord(prev_row - 1, prev_col), coord(prev_row - 1, col))
                } else {
                    (None, None)
                };

                let (al, cl) = if prev_col > 1 {
                    (coord(prev_row, prev_col - 1), coord(row, prev_col - 1))
                } else {
                    (None, None)
                };

                let (cd, dd) = if row < samples - 1 {
                    (coord(row + 1, prev_col), coord(row + 1, col))
                } else {
                    (None, None)
                };

                let (br, dr) = if col < samples - 1 {
                    (coord(prev_row, col + 1), coord(row, col + 1))
                } else {
                    (None, None)
                };

                let a_pos = a.position();
                let b_pos = b.position();
                let c_pos = c.position();
                let d_pos = d.position();

                //   au bu
                // al a b br
                // cl c d dr
                //   cd dd

                let a_int = intensity_intense(&a_pos, &c_pos, &b_pos, &au, &al);
                let b_int = intensity_intense(&b_pos, &a_pos, &d_pos, &br, &bu);
                let c_int = intensity_intense(&c_pos, &d_pos, &a_pos, &cl, &cd);
                let d_int = intensity_intense(&d_pos, &b_pos, &c_pos, &dd, &dr);

                let abc = {
                    let low = a.elevation.min(b.elevation).min(c.elevation);
                    let high = a.elevation.max(b.elevation).max(c.elevation);

                    (
                        a_pos,
                        b_pos.duplicate(),
                        c_pos.duplicate(),
                        (
                            a_int,
                            b_int,
                            c_int,
                        ),
                        rgb(low, high),
                    )
                };

                let bcd = {
                    let low = b.elevation.min(c.elevation).min(d.elevation);
                    let high = b.elevation.max(c.elevation).max(d.elevation);

                    (
                        b_pos,
                        c_pos,
                        d_pos,
                        (
                            b_int,
                            c_int,
                            d_int,
                        ),
                        rgb(low, high),
                    )
                };

                Some((abc, bcd))
            };

            let level_factor = 2u16.pow(level as u32);
            let mut tiles = Vec::with_capacity(samples as usize);
            let mut row = 1 + level_factor;
            while row < samples {
                let mut tiles_row = Vec::with_capacity(samples as usize);
                let mut col = 1 + level_factor;
                while col < samples {
                    tiles_row.push(cell_map(row - level_factor, col - level_factor, row, col));
                    col += level_factor;
                }
                tiles.push(tiles_row.into_boxed_slice());
                row += level_factor;
            }
            // println!("level {} factor {} tiles {}", level, level_factor, tiles.len());
            tiles_levels.push(tiles.into_boxed_slice());
        }

        Self {
            file,
            tiles_levels: tiles_levels.into_boxed_slice(),
        }
    }

    fn triangles(&self, bounds_levels: &[(f64, f64, f64, f64)], triangles: &mut Vec<HgtTriangle<'r, R>>) {
        let samples = self.file.resolution.samples();

        let min = self.file.coordinate(1, 1).unwrap();
        let max = self.file.coordinate(samples - 1, samples - 1).unwrap();

        let mut prev_opt: Option<(u16, u16, u16, u16)> = None;
        for (level, bounds) in bounds_levels.iter().enumerate() {
            if let Some(tiles) = self.tiles_levels.get(level) {
                let level_factor = 2u16.pow(level as u32);

                let start = self.file.position(bounds.0.min(max.0).max(min.0), bounds.1.min(max.1).max(min.1)).unwrap();
                let end = self.file.position(bounds.2.min(max.0).max(min.0), bounds.3.min(max.1).max(min.1)).unwrap();

                let start_row = cmp::max(1, start.0 / level_factor);
                let start_col = cmp::max(1, start.1 / level_factor);
                let end_row = cmp::min(tiles.len() as u16 - 1, end.0 / level_factor);
                let end_col = cmp::min(tiles[0].len() as u16 - 1, end.1 / level_factor);

                if end_row > start_row && end_col > start_col {
                    let mut push = |triangle: &HgtTriangle<'r, R>| {
                        triangles.push((
                            triangle.0.duplicate(),
                            triangle.1.duplicate(),
                            triangle.2.duplicate(),
                            triangle.3,
                            triangle.4
                        ));
                    };

                    for row in start_row..end_row + 1 {
                        if let Some(tiles_row) = tiles.get(row as usize) {
                            for col in start_col..end_col + 1 {
                                if let Some(prev) = prev_opt {
                                    // Skip if area was previously handled
                                    if row * level_factor > prev.0
                                    && row * level_factor < prev.2
                                    && col * level_factor > prev.1
                                    && col * level_factor < prev.3
                                    { continue; }
                                }
                                if let Some(tile_opt) = tiles_row.get(col as usize) {
                                    if let Some(tile) = tile_opt {
                                        push(&tile.0);
                                        push(&tile.1);
                                    }
                                } else {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }

                prev_opt = Some((
                    start_row * level_factor,
                    start_col * level_factor,
                    end_row * level_factor,
                    end_col * level_factor
                ));
            }
        }
    }
}

fn hgt_nearby_files<R: Spheroid + Send + Sync>(cache: &Arc<HgtCache>, latitude: f64, longitude: f64, res: HgtResolution, reference: &'static R, ground_color: Color, ocean_color: Color, levels: usize, hgt_files: &Arc<DashMap<(i16, i16), HgtFileTiles<'static, R>>>) {
    let f = latitude.floor();
    let l = longitude.floor();
    for &coord in &[
        (f, l),
        (f - 1.0, l),
        (f, l - 1.0),
        (f + 1.0, l),
        (f, l + 1.0),
        /* Diagonals
        (f - 1.0, l - 1.0),
        (f - 1.0, l + 1.0),
        (f + 1.0, l - 1.0),
        (f + 1.0, l + 1.0),
        */
    ] {
        let index = (coord.0 as i16, coord.1 as i16);
        if ! hgt_files.contains_key(&index) {
            //TODO: prevent reloading
            println!("loading {:?}", index);

            let cache = cache.clone();
            let hgt_files = hgt_files.clone();
            thread::spawn(move || {
                let hgt_file = HgtFileTiles::new(
                    cache.get(coord.0, coord.1, res).unwrap(),
                    reference,
                    ground_color,
                    ocean_color,
                    levels
                );
                hgt_files.insert(index, hgt_file);
                println!("loaded {:?}", index);
            });
        }
    }
}


fn main() {
    let mut w = Window::new_flags(-1, -1, 1024, 768, "FRIAR", &[WindowFlag::Async, WindowFlag::Resizable]).unwrap();

    let hud_color = Color::rgb(0x5F, 0xFF, 0x7F);
    let sky_color = Color::rgb(0x00, 0xBF, 0xFF);
    let ocean_color = Color::rgb(0x1C, 0x6B, 0xA0);
    let ground_color = Color::rgb(0x7A, 0x79, 0x4C);

    let hud_font = Font::from_path("res/fonts/RobotoMono/RobotoMono-Regular.ttf").unwrap();
    let mut hud_cache = FontCache::new(&hud_font, 24.0);
    let mut hud_string = String::with_capacity(16);

    let _ = write!(
        WindowWriter::new(
            &mut w,
            0, 0,
            hud_color
        ),
        "Loading"
    );

    w.sync();

    static earth: Earth = Earth;

    let mut gdl90 = Gdl90::new().unwrap();

    let mut xplane_opt: Option<XPlane> = None; //Some(XPlane::new("127.0.0.1", 30).unwrap());

    let (center_lat, center_lon, center_res): (f64, f64, bool) = if let Some(ref mut xplane) = xplane_opt {
        loop {
            if let Some(position) = xplane.position().unwrap() {
                break (position.latitude, position.longitude, false);
            } else {
                thread::sleep(Duration::new(0, 1000000000/1000));
            }
        }
    } else {
        (
            //37.619268, -112.166357, true // Bryce Canyon
            //39.610061, -106.056893, true // Dillon Reservoir
            //39.588303, -105.643829, true // Mount Evans
            //39.739230, -104.987403, true // Downtown Denver
            39.639720, -104.854705, true // Cherry Creek Reservoir
            //39.856096, -104.673727, true // Denver International Airport
            //46.306415, 7.791639, false // Raron
            //40.573420, 14.297834, false // Capri
            //40.821181, 14.426308, false // Mount Vesuvius
            //40.633537, 14.602547, false // Amalfi
            //27.988140, 86.925178, false // Mount Everest
            //38.876495, 100.006865, false // Zhangye Danxia
            //29.315328, 110.434751, false // Zhangjiajie
            //30.242933, 120.150318, false // Xihu
            //35.363486, 138.730486, false // Mount Fuji
        )
    };

    let (hgt_res, hgt_horizons) = if center_res {
        (HgtResolution::One, [2_000.0, 8_000.0, 32_000.0])
    } else {
        (HgtResolution::Three, [4_000.0, 16_000.0, 64_000.0])
    };

    let hgt_cache = Arc::new(HgtCache::new("cache"));

    let hgt_files: Arc<DashMap<(i16, i16), HgtFileTiles<Earth>>> = Arc::new(DashMap::new());

    hgt_nearby_files(&hgt_cache, center_lat, center_lon, hgt_res, &earth, ground_color, ocean_color, hgt_horizons.len(), &hgt_files);

    while hgt_files.get(&(center_lat.floor() as i16, center_lon.floor() as i16)).is_none() {
        let mut found_event = true;
        while found_event {
            found_event = false;
            for event in w.events() {
                found_event = true;
                match event.to_option() {
                    EventOption::Quit(_quit_event) => return,
                    _ => ()
                }
            }
        }
        w.set(Color::rgb(0, 0, 0));
        let _ = write!(
            WindowWriter::new(
                &mut w,
                0, 0,
                hud_color
            ),
            "Loading"
        );
        w.sync();
        thread::sleep(Duration::from_millis(10));
    }

    let ground = if let Some(hgt_file) = hgt_files.get(&(center_lat.floor() as i16, center_lon.floor() as i16)) {
        if let Some((row, col)) = hgt_file.file.position(center_lat, center_lon) {
            hgt_file.file.get(row, col).unwrap_or(0) as f64
        } else {
            0.0f64
        }
    } else {
        0.0f64
    };

    let center = earth.coordinate(center_lat, center_lon, ground);
    let orientation = (90.0f64, -20.0f64, 0.0f64);
    let original_fov = 90.0f64;
    let origin = center.offset(-2000.0, orientation.0, orientation.1);

    let osm_horizon = 4000.0;
    let osm_sw = center.offset(osm_horizon, 225.0, 0.0);
    let osm_ne = center.offset(osm_horizon, 45.0, 0.0);

    println!("Center: {}", center);
    println!("Origin: {}", origin);
    println!("Orientation: {:?}", orientation);
    println!("FOV: {}", original_fov);
    println!("OSM: {},{},{},{}", osm_sw.longitude, osm_sw.latitude, osm_ne.longitude, osm_ne.latitude);

    let runways = oap_runways(&earth);

    println!("Runways: {}", runways.len());

    let osm_file_opt: Option<&'static str> = {
        None
        //Some("res/planet_-104.99279,39.73659_-104.98198,39.74187.osm.pbf")
        //Some("cache/OSM/Denver.osm.pbf")
    };
    let mut osm_opt = osm_file_opt.and_then(|osm_file| match Osm::new(&osm_file) {
        Ok(osm) => {
            println!("Nodes: {}, Ways: {}", osm.nodes.len(), osm.ways.len());
            Some(osm)
        },
        Err(err) => {
            println!("Failed to read {}: {}", osm_file, err);
            None
        }
    });

    let mut hgt_triangles = Vec::new();
    let mut osm_triangles = Vec::new();
    let mut oap_triangles = Vec::new();

    let mut traffic_ownship_alt = None;
    let mut traffics = HashMap::new();
    let mut traffic_triangles = Vec::new();

    let mut intersect_heading = 0.0;
    let mut intersect_pitch = 0.0;
    let mut intersect_opt = None;
    let mut intersect_triangles = Vec::new();

    let mut viewer = origin.duplicate();
    let mut heading = orientation.0;
    let mut pitch = orientation.1;
    let mut roll = orientation.2;
    let mut fov = original_fov;

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

    let mut zoom_in = false;
    let mut zoom_out = false;
    let mut boresight_intersect = false;
    let mut mouse_intersect = false;

    let mut shift = false;

    let mut mouse_x = 0;
    let mut mouse_y = 0;

    let mut debug = false;
    let mut retraffic = false;
    let mut reintersect = false;
    let mut rehgt = true;
    let mut redraw = true;
    let mut redraw_times = 2;
    let mut fill = true;
    let mut hud = true;
    let mut z_buffer = vec![0.0; (w.width() * w.height()) as usize];
    let mut row_mutexes = Vec::with_capacity(w.height() as usize);
    for _ in 0..w.height() {
        row_mutexes.push(Mutex::new(()));
    }
    let mut triangles = Vec::with_capacity(hgt_triangles.len() + osm_triangles.len() + oap_triangles.len() + traffic_triangles.len() + intersect_triangles.len());

    let mut last_instant = Instant::now();
    loop {
        let instant = Instant::now();
        let duration = instant.duration_since(last_instant);
        last_instant = instant;
        let time = duration.as_secs() as f64 + duration.subsec_nanos() as f64 / 1000000000.0;
        let speed = if shift { 1000.0 } else { 250.0 } * time;
        let speed_rot = fov * time;
        let speed_zoom = 30.0 * time;

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
                            rehgt = true;
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
                            heading = orientation.0;
                            pitch = orientation.1;
                            roll = orientation.2;
                            redraw = true;
                        },

                        orbclient::K_Z => {
                            zoom_in = key_event.pressed;
                        },
                        orbclient::K_X => {
                            zoom_out = key_event.pressed;
                        },
                        orbclient::K_C if key_event.pressed => {
                            fov = original_fov;
                            redraw = true;
                        },
                        orbclient::K_SPACE => {
                            boresight_intersect = key_event.pressed;
                        }

                        orbclient::K_LEFT_SHIFT => {
                            shift = key_event.pressed;
                            redraw = true;
                        },

                        orbclient::K_B if key_event.pressed => {
                            debug = !debug;
                            redraw = true;
                        },
                        orbclient::K_F if key_event.pressed => {
                            fill = !fill;
                            redraw = true;
                        },
                        orbclient::K_H if key_event.pressed => {
                            hud = !hud;
                            redraw = true;
                        },

                        _ => (),
                    },
                    EventOption::Mouse(mouse_event) => {
                        mouse_x = mouse_event.x;
                        mouse_y = mouse_event.y;
                    },
                    EventOption::Button(button_event) => {
                        mouse_intersect = button_event.left;
                    }
                    EventOption::Resize(resize_event) => {
                        w.sync_path();
                        z_buffer = vec![0.0; (resize_event.width * resize_event.height) as usize];
                        let mut row_mutexes = Vec::with_capacity(resize_event.height as usize);
                        for _ in 0..resize_event.height {
                            row_mutexes.push(Mutex::new(()));
                        }
                        redraw = true;
                    },
                    EventOption::Quit(_quit_event) => return,
                    _ => ()
                }
            }
        }

        {
            if move_forward {
                viewer = viewer.offset(speed, heading, pitch);
                rehgt = true;
            }

            if move_aft {
                viewer = viewer.offset(-speed, heading, pitch);
                rehgt = true;
            }

            if move_left {
                viewer = viewer.offset(-speed, heading + 90.0, 0.0);
                rehgt = true;
            }

            if move_right {
                viewer = viewer.offset(speed, heading + 90.0, 0.0);
                rehgt = true;
            }

            if move_up {
                viewer = viewer.offset(speed, heading, pitch + 90.0);
                rehgt = true;
            }

            if move_down {
                viewer = viewer.offset(-speed, heading, pitch + 90.0);
                rehgt = true;
            }

            if rotate_left {
                heading = (heading - speed_rot * roll.to_radians().cos()).rem_euclid(360.0);
                pitch = (pitch - speed_rot * roll.to_radians().sin()).rem_euclid(360.0);
                redraw = true;
            }

            if rotate_right {
                heading = (heading + speed_rot * roll.to_radians().cos()).rem_euclid(360.0);
                pitch = (pitch + speed_rot * roll.to_radians().sin()).rem_euclid(360.0);
                redraw = true;
            }

            if rotate_up {
                heading = (heading - speed_rot * roll.to_radians().sin()).rem_euclid(360.0);
                pitch = (pitch + speed_rot * roll.to_radians().cos()).rem_euclid(360.0);
                redraw = true;
            }

            if rotate_down {
                heading = (heading + speed_rot * roll.to_radians().sin()).rem_euclid(360.0);
                pitch = (pitch - speed_rot * roll.to_radians().cos()).rem_euclid(360.0);
                redraw = true;
            }

            if roll_left {
                roll = (roll + speed_rot).rem_euclid(360.0);
                redraw = true;
            }

            if roll_right {
                roll = (roll - speed_rot).rem_euclid(360.0);
                redraw = true;
            }

            if zoom_in {
                fov = (fov - speed_zoom).max(1.0);
                redraw = true;
            }

            if zoom_out {
                fov = (fov + speed_zoom).min(180.0);
                redraw = true;
            }

            if boresight_intersect {
                intersect_heading = heading;
                intersect_pitch = pitch;
                reintersect = true;
            }

            if mouse_intersect {
                let w_w = w.width() as f64;
                let w_h = w.height() as f64;

                let a = w_w.max(w_h);
                let ax = a / w_w;
                let ay = a / w_h;

                let x = (2.0 * ((mouse_x as f64) / w_w) - 1.0) / ax;
                let y = (2.0 * ((mouse_y as f64) / w_h) - 1.0) / ay;

                let t = -roll.to_radians();
                let ct = t.cos();
                let st = t.sin();

                let bx = x * ct - y * st;
                let by = y * ct + x * st;
                let bz = 1.0/(fov.to_radians()/2.0).tan();

                let display = viewer.offset(bz, heading, pitch);
                let display_pos = display.position();

                let up = display.offset(1.0, heading, pitch + 90.0);
                let up_pos = up.position();
                let up_vec = display_pos.vector(&up_pos);
                let up_unit = up_vec.normalize();

                let right = display.offset(1.0, heading + 90.0, 0.0);
                let right_pos = right.position();
                let right_vec = display_pos.vector(&right_pos);
                let right_unit = right_vec.normalize();

                let mouse_vec = right_unit.multiply(bx).add(&up_unit.multiply(-by));
                let mouse_pos = {
                    let v = display_pos.to_vector().add(&mouse_vec);
                    earth.position(v.x, v.y, v.z)
                };

                let mouse_coord = mouse_pos.coordinate();

                intersect_heading = viewer.heading(&mouse_coord);
                intersect_pitch = viewer.pitch(&mouse_coord);
                reintersect = true;
            }
        }

        while let Some(msg) = gdl90.message().unwrap() {
            if let Some(kind) = msg.kind() {
                match kind {
                    Gdl90Kind::Ownship(traffic) => {
                        println!("{:?} ({}): {}, {}, {}, {}",
                            traffic.address(),
                            traffic.callsign(),
                            traffic.latitude(),
                            traffic.longitude(),
                            traffic.altitude(),
                            traffic.heading()
                        );

                        traffics.insert(traffic.id(), (traffic, traffic_ownship_alt));

                        retraffic = true;
                    },
                    Gdl90Kind::GeoAltitude(altitude) => {
                        traffic_ownship_alt = Some(altitude.altitude());
                    },
                    Gdl90Kind::Traffic(traffic) => {
                        println!("{:?} ({}): {}, {}, {}, {}",
                            traffic.address(),
                            traffic.callsign(),
                            traffic.latitude(),
                            traffic.longitude(),
                            traffic.altitude(),
                            traffic.heading()
                        );

                        traffics.insert(traffic.id(), (traffic, None));

                        retraffic = true;
                    },
                    _ => ()
                }
            }
        }

        if let Some(ref mut xplane) = xplane_opt {
            Timer::new("xplane", debug);
            while let Some(position) = xplane.position().unwrap() {
                // println!("{:#?}", position);

                viewer = earth.coordinate(
                    position.latitude,
                    position.longitude,
                    position.elevation
                );

                heading = position.heading as f64;
                pitch = position.pitch as f64;
                roll = -position.roll as f64;

                rehgt = true;
            }
        }

        if retraffic {
            let timer = Timer::new("traffic", debug);

            retraffic = false;

            traffic_triangles.clear();
            for (id, (traffic, alt_override)) in &traffics {
                let alt = if let Some(some) = alt_override {
                    *some
                } else {
                    traffic.altitude()
                };

                let traffic_coord = earth.coordinate(
                    traffic.latitude(),
                    traffic.longitude(),
                    alt * 0.3048
                );

                let traffic_heading = traffic.heading();

                let size = 10.0;
                let angle = 30.0;
                // Top Left
                let a = traffic_coord.offset(size, traffic_heading + 180.0 + angle, angle);
                // Top Right
                let b = traffic_coord.offset(size, traffic_heading + 180.0 - angle, angle);
                // Bottom Right
                let c = traffic_coord.offset(size, traffic_heading + 180.0 - angle, -angle);
                // Bottom Left
                let d = traffic_coord.offset(size, traffic_heading + 180.0 + angle, -angle);

                let traffic_pos = traffic_coord.position();
                let a_pos = a.position();
                let b_pos = b.position();
                let c_pos = c.position();
                let d_pos = d.position();

                let rgb = (hud_color.r(), hud_color.g(), hud_color.b());

                // Draw rear
                {
                    traffic_triangles.push((
                        a_pos.duplicate(),
                        b_pos.duplicate(),
                        d_pos.duplicate(),
                        (0.5, 0.5, 0.5),
                        rgb,
                    ));

                    traffic_triangles.push((
                        b_pos.duplicate(),
                        d_pos.duplicate(),
                        c_pos.duplicate(),
                        (0.5, 0.5, 0.5),
                        rgb,
                    ));
                }

                // Draw sides
                {
                    traffic_triangles.push((
                        traffic_pos.duplicate(),
                        a_pos.duplicate(),
                        b_pos.duplicate(),
                        (1.0, 1.0, 1.0),
                        rgb,
                    ));

                    traffic_triangles.push((
                        traffic_pos.duplicate(),
                        b_pos.duplicate(),
                        c_pos.duplicate(),
                        (1.0, 1.0, 1.0),
                        rgb,
                    ));

                    traffic_triangles.push((
                        traffic_pos.duplicate(),
                        c_pos.duplicate(),
                        d_pos.duplicate(),
                        (1.0, 1.0, 1.0),
                        rgb,
                    ));

                    traffic_triangles.push((
                        traffic_pos.duplicate(),
                        d_pos.duplicate(),
                        a_pos.duplicate(),
                        (1.0, 1.0, 1.0),
                        rgb,
                    ));
                }
            }

            redraw = true;

            drop(timer);
        }

        if reintersect {
            let timer = Timer::new("intersect", debug);

            reintersect = false;

            intersect_opt = if let Some(hgt_file) = hgt_files.get(&(viewer.latitude.floor() as i16, viewer.longitude.floor() as i16)) {
                hgt_intersect(&hgt_file.file, &earth, &viewer, intersect_heading, intersect_pitch)
            } else {
                None
            };

            intersect_triangles.clear();
            if let Some(ref intersect) = intersect_opt {
                let size = 10.0;
                let angle = 60.0;
                // NW
                let a = intersect.offset(size, 315.0, angle);
                // NE
                let b = intersect.offset(size, 45.0, angle);
                // SE
                let c = intersect.offset(size, 135.0, angle);
                // SW
                let d = intersect.offset(size, 225.0, angle);

                let intersect_pos = intersect.position();
                let a_pos = a.position();
                let b_pos = b.position();
                let c_pos = c.position();
                let d_pos = d.position();

                let rgb = (hud_color.r(), hud_color.g(), hud_color.b());

                // Draw top
                {
                    intersect_triangles.push((
                        a_pos.duplicate(),
                        b_pos.duplicate(),
                        d_pos.duplicate(),
                        (1.0, 1.0, 1.0),
                        rgb,
                    ));

                    intersect_triangles.push((
                        b_pos.duplicate(),
                        d_pos.duplicate(),
                        c_pos.duplicate(),
                        (1.0, 1.0, 1.0),
                        rgb,
                    ));
                }

                // Draw sides
                {
                    intersect_triangles.push((
                        intersect_pos.duplicate(),
                        a_pos.duplicate(),
                        b_pos.duplicate(),
                        (0.5, 0.5, 0.5),
                        rgb,
                    ));

                    intersect_triangles.push((
                        intersect_pos.duplicate(),
                        b_pos.duplicate(),
                        c_pos.duplicate(),
                        (0.5, 0.5, 0.5),
                        rgb,
                    ));

                    intersect_triangles.push((
                        intersect_pos.duplicate(),
                        c_pos.duplicate(),
                        d_pos.duplicate(),
                        (0.5, 0.5, 0.5),
                        rgb,
                    ));

                    intersect_triangles.push((
                        intersect_pos.duplicate(),
                        d_pos.duplicate(),
                        a_pos.duplicate(),
                        (0.5, 0.5, 0.5),
                        rgb,
                    ));
                }
            }

            redraw = true;

            drop(timer);
        }

        let viewer_pos = viewer.position();

        if rehgt {
            let timer = Timer::new("hgt", debug);

            rehgt = false;

            let mut bounds_levels = Vec::with_capacity(hgt_horizons.len());
            for hgt_horizon in hgt_horizons.iter() {
                let viewer_sw = viewer.offset(*hgt_horizon, 225.0, 0.0);
                let viewer_ne = viewer.offset(*hgt_horizon, 45.0, 0.0);
                let bounds = (
                    viewer_sw.latitude,
                    viewer_sw.longitude,
                    viewer_ne.latitude,
                    viewer_ne.longitude
                );
                // println!("bounds {}: {:?}", hgt_horizon, bounds);
                bounds_levels.push(bounds);
            }

            if hgt_files.get(&(viewer.latitude.floor() as i16, viewer.longitude.floor() as i16)).is_none() {
                hgt_nearby_files(&hgt_cache, viewer.latitude, viewer.longitude, hgt_res, &earth, ground_color, ocean_color, hgt_horizons.len(), &hgt_files);

                while hgt_files.get(&(viewer.latitude.floor() as i16, viewer.longitude.floor() as i16)).is_none() {
                    let mut found_event = true;
                    while found_event {
                        found_event = false;
                        for event in w.events() {
                            found_event = true;
                            match event.to_option() {
                                EventOption::Quit(_quit_event) => return,
                                _ => ()
                            }
                        }
                    }
                    w.set(Color::rgb(0, 0, 0));
                    let _ = write!(
                        WindowWriter::new(
                            &mut w,
                            0, 0,
                            hud_color
                        ),
                        "Loading"
                    );
                    w.sync();
                    thread::sleep(Duration::from_millis(10));
                }
            }

            hgt_triangles.clear();
            for hgt_file in hgt_files.iter() {
                hgt_file.triangles(&bounds_levels, &mut hgt_triangles);
            }

            if let Some(ref osm) = osm_opt.take() { //TODO: Improve performance
                osm_triangles.clear();
                osm_way_triangles(osm, &earth, bounds_levels[bounds_levels.len() - 1], ground, &mut osm_triangles)
            }

            oap_triangles.clear();
            oap_runway_triangles(&runways, bounds_levels[bounds_levels.len() - 1], &mut oap_triangles);

            redraw = true;

            drop(timer);
        }

        redraw = true; // Force redraw
        if redraw {
            let timer = Timer::new("draw", debug);

            if redraw_times > 0 {
                redraw_times -= 1;
            } else {
                redraw = false;
            }

            let viewer_rot = viewer.rotation();

            let ground_perspective = viewer_pos.perspective(viewer_rot.0, viewer_rot.1, viewer_rot.2);
            let ground_pos = ground_perspective.position(0.0, 0.0, 0.0);

            let perspective = ground_pos.perspective(pitch + 90.0, 0.0, heading - 90.0);
            let viewport = perspective.viewport(0.0, 0.0, 1.0/(fov.to_radians()/2.0).tan());

            let w_w = w.width() as i32;
            let w_h = w.height() as i32;
            let screen = viewport.screen(w_w as f64, w_h as f64, roll);

            let clip = (
                -0.1 * screen.x, 1.1 * screen.x,
                -0.1 * screen.y, 1.1 * screen.y,
                0.0001
            );
            let clip_valid = |point: &(f64, f64, f64)| {
                point.0 > clip.0 && point.0 < clip.1 &&
                point.1 > clip.2 && point.1 < clip.3 &&
                point.2 > clip.4
            };
            let triangle_map = |triangle: &(Position<Earth>, Position<Earth>, Position<Earth>, (f32, f32, f32), (u8, u8, u8))| -> Option<(Triangle, Color)> {

                let a_earth = &triangle.0;
                let a_ground = ground_perspective.transform(a_earth);
                let a_screen = screen.transform(&a_ground);
                if ! clip_valid(&a_screen) { return None; }

                let b_earth = &triangle.1;
                let b_ground = ground_perspective.transform(b_earth);
                let b_screen = screen.transform(&b_ground);
                if ! clip_valid(&b_screen) { return None; }

                let c_earth = &triangle.2;
                let c_ground = ground_perspective.transform(c_earth);
                let c_screen = screen.transform(&c_ground);
                if ! clip_valid(&c_screen) { return None; }

                let a_dist = viewer_pos.vector(&a_earth).norm() as f32;
                let b_dist = viewer_pos.vector(&b_earth).norm() as f32;
                let c_dist = viewer_pos.vector(&c_earth).norm() as f32;

                let a = Point {
                    x: a_screen.0 as i32,
                    y: a_screen.1 as i32,
                    z: 1.0 / a_dist,
                    intensity: (triangle.3).0,
                };

                let b = Point {
                    x: b_screen.0 as i32,
                    y: b_screen.1 as i32,
                    z: 1.0 / b_dist,
                    intensity: (triangle.3).1,
                };

                let c = Point {
                    x: c_screen.0 as i32,
                    y: c_screen.1 as i32,
                    z: 1.0 / c_dist,
                    intensity: (triangle.3).2,
                };

                let (cr, cg, cb) = (
                    (triangle.4).0,
                    (triangle.4).1,
                    (triangle.4).2
                );

                Some((
                    Triangle::new(a, b, c),
                    Color::rgb(cr, cg, cb),
                ))
            };

            {
                let timer = Timer::new("transform", debug);

                triangles.clear();
                triangles.par_extend(
                    hgt_triangles.par_iter()
                        .chain(osm_triangles.par_iter())
                        .chain(oap_triangles.par_iter())
                        .chain(traffic_triangles.par_iter())
                        .chain(intersect_triangles.par_iter())
                        .filter_map(triangle_map)
                );

                drop(timer);
            }

            w.set(sky_color);

            for i in 0..z_buffer.len() {
                z_buffer[i] = 0.0;
            }

            let draw_horizon = false;
            if draw_horizon {
                let viewer_on_ground = earth.coordinate(viewer.latitude, viewer.longitude, 0.0);

                let radius = viewer_on_ground.radius();
                let dist = (viewer.elevation * (2.0 * radius + viewer.elevation)).sqrt();

                for &d in &[dist, -dist] {
                    let horizon_coord = viewer_on_ground.offset(d, heading, 0.0);
                    let horizon_earth = horizon_coord.position();
                    let horizon_ground = ground_perspective.transform(&horizon_earth);
                    let horizon_screen = screen.transform(&horizon_ground);

                    let yl = horizon_screen.1 - horizon_screen.0 * roll.to_radians().tan();
                    let yr = horizon_screen.1 + ((w_w as f64) - horizon_screen.0) * roll.to_radians().tan();

                    if horizon_screen.2.is_sign_positive() {
                        if d.is_sign_positive() {
                            let y = yl.max(yr).round().max(0.0).min(screen.y) as i32;
                            w.rect(0, y, w_w as u32, (w_h - y as i32) as u32, ground_color);
                        } else {
                            let y = yl.min(yr).round().max(0.0).min(screen.y) as i32;
                            w.rect(0, 0, w_w as u32, y as u32, ground_color);
                        }

                        // println!("{}: {}, {}", roll, yl, yr);

                        let flip = if d.is_sign_positive() {
                            roll > 90.0 && roll < 270.0
                        } else {
                            !(roll > 90.0 && roll < 270.0)
                        };

                        let (xl, xr) = (0, w_w - 1);

                        let a = Point {
                            x: xl,
                            y: yl.round() as i32,
                            z: 1.0 / (dist as f32),
                            intensity: 1.0
                        };

                        let b = Point {
                            x: xr,
                            y: yr.round() as i32,
                            z: 1.0 / (dist as f32),
                            intensity: 1.0
                        };

                        let c = if yl > yr {
                            Point {
                                x: xr,
                                y: yl.round() as i32,
                                z: 1.0 / (dist as f32),
                                intensity: 1.0
                            }
                        } else {
                            Point {
                                x: xl,
                                y: yr.round() as i32,
                                z: 1.0 / (dist as f32),
                                intensity: 1.0
                            }
                        };

                        let triangle = Triangle::new(a, b, c);
                        triangle.fill(&mut w, &mut z_buffer, ground_color);
                    }
                }
            }

            {
                let timer = Timer::new("fill", debug);

                if fill {
                    if true {
                        let data_addr = w.data_mut().as_mut_ptr() as usize;
                        let z_buffer_addr = z_buffer.as_mut_ptr() as usize;
                        triangles.par_iter().for_each(|(triangle, color)| {
                            triangle.par_fill(
                                w_w, w_h,
                                data_addr as *mut Color,
                                z_buffer_addr as *mut f32,
                                *color,
                                &row_mutexes,
                            );
                        });
                    } else {
                        for (triangle, color) in triangles.iter() {
                            triangle.fill(
                                &mut w,
                                &mut z_buffer,
                                *color,
                            );
                        }
                    }
                } else {
                    for (triangle, color) in triangles.iter() {
                        triangle.draw(&mut w, *color);
                    }
                }

                drop(timer);
            }

            if hud {
                let timer = Timer::new("hud", debug);

                let mut h = 0;
                while h < 360 {
                    let h_coord = viewer.offset(1.0, h as f64, 0.0);
                    let h_earth = h_coord.position();
                    let h_ground = ground_perspective.transform(&h_earth);
                    let h_screen = screen.transform(&h_ground);

                    if clip_valid(&h_screen) {
                        let size = 16.0;

                        let r = roll.to_radians();
                        let rc = r.cos();
                        let rs = r.sin();

                        let dx = size * rs;
                        let dy = size * rc;

                        line_f64(
                            &mut w,
                            h_screen.0,
                            h_screen.1,
                            h_screen.0 - dx,
                            h_screen.1 + dy,
                            hud_color
                        );

                        hud_string.clear();
                        let _ = write!(
                            hud_string,
                            "{}",
                            h/10
                        );

                        let text = hud_cache.render(&hud_string);
                        text.draw(
                            &mut w,
                            (h_screen.0.round() as i32) - (text.width() as i32)/2,
                            (h_screen.1.round() as i32) - (text.height() as i32),
                            hud_color
                        );
                    }

                    h += 10;
                }

                let mut p = 0;
                while p < 360 {
                    let p_coord = viewer.offset(1.0, heading, p as f64);
                    let p_earth = p_coord.position();
                    let p_ground = ground_perspective.transform(&p_earth);
                    let p_screen = screen.transform(&p_ground);

                    if clip_valid(&p_screen) {
                        let p_draw = if p <= 90 {
                            p
                        } else if p <= 270 {
                            180 - p
                        } else {
                            p - 360
                        };

                        let size = if p_draw == 0 {
                            128.0
                        } else {
                            64.0
                        };

                        let hole = if p_draw == 0 {
                            0.0
                        } else {
                            24.0
                        };

                        let r = roll.to_radians();
                        let rc = r.cos();
                        let rs = r.sin();

                        let dx = size * rc;
                        let dy = size * rs;
                        let hx = hole * rc;
                        let hy = hole * rs;

                        line_f64(
                            &mut w,
                            p_screen.0 - dx,
                            p_screen.1 - dy,
                            p_screen.0 - hx,
                            p_screen.1 - hy,
                            hud_color
                        );

                        line_f64(
                            &mut w,
                            p_screen.0 + hx,
                            p_screen.1 + hy,
                            p_screen.0 + dx,
                            p_screen.1 + dy,
                            hud_color
                        );

                        if p_draw != 0 {
                            hud_string.clear();
                            let _ = write!(
                                hud_string,
                                "{}",
                                p_draw
                            );

                            let text = hud_cache.render(&hud_string);
                            text.draw(
                                &mut w,
                                (p_screen.0.round() as i32) - (text.width() as i32)/2,
                                (p_screen.1.round() as i32) - (text.height() as i32)/2,
                                hud_color
                            );
                        }
                    }

                    p += 5;
                }

                let center = (w_w/2 as i32, w_h/2 as i32);
                w.line(center.0 - 5, center.1, center.0 + 5, center.1, hud_color);
                w.line(center.0, center.1 - 5, center.0, center.1 + 5, hud_color);

                for runway in &runways {
                    let runway_pos = runway.coord.position();
                    let runway_dist = viewer_pos.vector(&runway_pos).norm();

                    if runway_dist < hgt_horizons[hgt_horizons.len() - 1] {
                        let runway_ground = ground_perspective.transform(&runway_pos);
                        let runway_screen = screen.transform(&runway_ground);

                        if clip_valid(&runway_screen) {
                            let x = runway_screen.0.round() as i32;
                            let y = runway_screen.1.round() as i32;
                            w.circle(x, y, 20, hud_color);

                            {
                                let end = runway.coord.offset(runway.length, runway.heading, 0.0);
                                let end_pos = end.position();
                                let end_ground = ground_perspective.transform(&end_pos);
                                let end_screen = screen.transform(&end_ground);

                                if clip_valid(&end_screen) {
                                    let xe = end_screen.0.round() as i32;
                                    let ye = end_screen.1.round() as i32;
                                    w.line(x, y, xe, ye, hud_color);
                                }
                            }

                            {
                                hud_string.clear();
                                let _ = write!(
                                    hud_string,
                                    "{} {}",
                                    runway.airport_ident,
                                    runway.ident
                                );

                                let text = hud_cache.render(&hud_string);
                                text.draw(
                                    &mut w,
                                    x - (text.width() as i32)/2,
                                    y - (text.height() as i32) - 20,
                                    hud_color
                                );
                            }

                            {
                                hud_string.clear();
                                let _ = write!(
                                    hud_string,
                                    "{}m",
                                    runway_dist.round() as u32
                                );

                                let text = hud_cache.render(&hud_string);
                                text.draw(
                                    &mut w,
                                    x - (text.width() as i32)/2,
                                    y + 20,
                                    hud_color
                                );
                            }
                        }
                    }
                }

                for (id, (traffic, alt_override)) in &traffics {
                    let alt = if let Some(some) = alt_override {
                        *some
                    } else {
                        traffic.altitude()
                    };

                    let traffic_coord = earth.coordinate(
                        traffic.latitude(),
                        traffic.longitude(),
                        alt * 0.3048
                    );
                    let traffic_pos = traffic_coord.position();
                    let traffic_ground = ground_perspective.transform(&traffic_pos);
                    let traffic_screen = screen.transform(&traffic_ground);

                    if clip_valid(&traffic_screen) {
                        let x = traffic_screen.0.round() as i32;
                        let y = traffic_screen.1.round() as i32;
                        w.circle(x, y, 20, hud_color);

                        {
                            let text = hud_cache.render(&traffic.callsign());
                            text.draw(
                                &mut w,
                                x - (text.width() as i32)/2,
                                y - (text.height() as i32) - 20,
                                hud_color
                            );
                        }

                        let traffic_dist = viewer_pos.vector(&traffic_pos).norm();

                        hud_string.clear();
                        let _ = write!(
                            hud_string,
                            "{}m",
                            traffic_dist.round() as u32
                        );

                        {
                            let text = hud_cache.render(&hud_string);
                            text.draw(
                                &mut w,
                                x - (text.width() as i32)/2,
                                y + 20,
                                hud_color
                            );
                        }
                    }
                }

                if let Some(ref intersect) = intersect_opt {
                    let intersect_pos = intersect.position();
                    let intersect_ground = ground_perspective.transform(&intersect_pos);
                    let intersect_screen = screen.transform(&intersect_ground);

                    if clip_valid(&intersect_screen) {
                        let x = intersect_screen.0.round() as i32;
                        let y = intersect_screen.1.round() as i32;
                        w.circle(x, y, 20, hud_color);

                        let intersect_dist = viewer_pos.vector(&intersect_pos).norm();

                        hud_string.clear();
                        let _ = write!(
                            hud_string,
                            "{}m",
                            intersect_dist.round() as u32
                        );

                        let text = hud_cache.render(&hud_string);
                        text.draw(
                            &mut w,
                            x - (text.width() as i32)/2,
                            y + 20,
                            hud_color
                        );
                    }
                }

                drop(timer);
            }

            if debug {
                let mut y = 0;

                let _ = write!(
                    WindowWriter::new(&mut w, 0, y, hud_color),
                    "Coord: {}",
                    viewer
                );
                y += 16;

                let _ = write!(
                    WindowWriter::new(&mut w, 0, y, hud_color),
                    "Pos: {}",
                    viewer_pos
                );
                y += 16;

                let _ = write!(
                    WindowWriter::new(&mut w, 0, y, hud_color),
                    "Rot: {}, {}, {}",
                    heading, pitch, roll
                );
                y += 16;

                let _ = write!(
                    WindowWriter::new(&mut w, 0, y, hud_color),
                    "FOV: {}",
                    fov
                );
                y += 16;

                let _ = write!(
                    WindowWriter::new(&mut w, 0, y, hud_color),
                    "Triangles (Hgt): {}",
                    hgt_triangles.len()
                );
                y += 16;

                let _ = write!(
                    WindowWriter::new(&mut w, 0, y, hud_color),
                    "Triangles (Osm): {}",
                    osm_triangles.len()
                );
                y += 16;

                let _ = write!(
                    WindowWriter::new(&mut w, 0, y, hud_color),
                    "Triangles (Oap): {}",
                    oap_triangles.len()
                );
                y += 16;

                let _ = write!(
                    WindowWriter::new(&mut w, 0, y, hud_color),
                    "Triangles (Drawn): {}",
                    triangles.len()
                );
                y += 16;

                let _ = write!(
                    WindowWriter::new(&mut w, 0, y, hud_color),
                    "FPS: {}",
                    1.0/time
                );
                y += 16;

                if let Some(ref intersect) = intersect_opt {
                    let _ = write!(
                        WindowWriter::new(&mut w, 0, y, hud_color),
                        "Looking at: {}",
                        intersect
                    );
                }
            }

            w.sync();

            drop(timer);
        } else {
            thread::sleep(Duration::new(0, 1000000000/1000));
        }
    }
}
