use std::{f64, fmt};

use crate::position::Position;
use crate::spheroid::Spheroid;

pub struct Coordinate<'r, R: Spheroid + 'r> {
    pub reference: &'r R,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
}

impl<'r, R: Spheroid> fmt::Display for Coordinate<'r, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.latitude, self.longitude, self.elevation)
    }
}

impl<'r, R: Spheroid> Coordinate<'r, R> {
    /// Create a Coordinate
    pub fn new(reference: &'r R, latitude: f64, longitude: f64, elevation: f64) -> Self {
        Self {
            reference,
            latitude,
            longitude,
            elevation,
        }
    }

    /// Duplicate this coordinate
    pub fn duplicate(&self) -> Self {
        Self::new(self.reference, self.latitude, self.longitude, self.elevation)
    }

    /// Radius of the spheroid plus elevation at given coordinate, in meters
    ///
    /// Adapted from https://en.wikipedia.org/wiki/Earth_radius#Location-dependent_radii
    pub fn radius(&self) -> f64 {
        let a = self.reference.radius_equatorial();
        let b = self.reference.radius_polar();
        let f = self.latitude.to_radians();

        (
            ((a.powi(2) * f.cos()).powi(2) + (b.powi(2) * f.sin()).powi(2))
            /
            ((a * f.cos()).powi(2) + (b * f.sin()).powi(2))
        ).sqrt() + self.elevation
    }

    /// Great-circle distance to another Coordinate in meters, using average radius at the two latitudes
    ///
    /// Adapted from https://en.wikipedia.org/wiki/Great-circle_distance#Computational_formulas
    pub fn distance(&self, to: &Self) -> f64 {
        let f1 = self.latitude.to_radians();
        let l1 = self.longitude.to_radians();
        let f2 = to.latitude.to_radians();
        let l2 = to.longitude.to_radians();

        let th = 2.0 * (
            ((f2 - f1)/2.0).sin().powi(2) + f1.cos() * f2.cos() * ((l2 - l1)/2.0).sin().powi(2)
        ).sqrt().asin();

        let r1 = self.radius();
        let r2 = to.radius();
        let r = (r1 + r2)/2.0;
        th*r
    }

    /// Great-circle heading to another Coordinate in degrees
    ///
    /// Adapted from https://en.wikipedia.org/wiki/Great-circle_navigation#Course
    pub fn heading(&self, to: &Self) -> f64 {
        let f1 = self.latitude.to_radians();
        let l1 = self.longitude.to_radians();
        let f2 = to.latitude.to_radians();
        let l2 = to.longitude.to_radians();

        (
            (l2 - l1).sin() * f2.cos()
        ).atan2(
            f1.cos() * f2.sin() - f1.sin() * f2.cos() * (l2 - l1).cos()
        ).rem_euclid(2.0 * f64::consts::PI).to_degrees()
    }

    pub fn pitch(&self, to: &Self) -> f64 {
        let d = self.distance(to);
        let e = to.elevation - self.elevation;
        let p = (e / (d.powi(2) + e.powi(2)).sqrt()).asin();
        p.to_degrees()
    }

    pub fn offset(&self, distance: f64, heading: f64, pitch: f64) -> Self {
        let f1 = self.latitude.to_radians();
        let l1 = self.longitude.to_radians();
        let h = heading.to_radians();
        let p = pitch.to_radians();
        let d = distance * p.cos() / self.radius();
        let e = distance * p.sin();

        let f2 = (
            f1.sin() * d.cos() + f1.cos() * d.sin() * h.cos()
        ).asin();
        let dl = (
            h.sin() * d.sin() * f1.cos()
        ).atan2(
            d.cos() - f1.sin() * f2.sin()
        );
        let l2 = (
            l1 + dl + f64::consts::PI
        ).rem_euclid(2.0 * f64::consts::PI) - f64::consts::PI;

        Self {
            reference: self.reference,
            latitude: f2.to_degrees(),
            longitude: l2.to_degrees(),
            elevation: self.elevation + e,
        }
    }

    /// Convert to Position
    ///
    /// Adapted from https://en.wikipedia.org/wiki/Geographic_coordinate_conversion#From_geodetic_to_ECEF_coordinates
    pub fn position(&self) -> Position<'r, R> {
        let a = self.reference.radius_equatorial();
        let b = self.reference.radius_polar();
        let f = self.latitude.to_radians();
        let l = self.longitude.to_radians();
        let h = self.elevation;
        let n = a.powi(2)
            /
            (
                a.powi(2) * f.cos().powi(2) + b.powi(2) * f.sin().powi(2)
            ).sqrt();

        let x = (n + h) * f.cos() * l.cos();
        let y = (n + h) * f.cos() * l.sin();
        let z = ((b.powi(2) / a.powi(2)) * n + h) * f.sin();

        Position::new(self.reference, x, y, z)
    }

    /// Get rotation of ground plane in ECEF
    pub fn rotation(&self) -> (f64, f64, f64) {
        let f = self.latitude.to_radians();
        let l = self.longitude.to_radians();

        let rx = 0.0f64;
        let ry = (f + f64::consts::PI/2.0).rem_euclid(2.0 * f64::consts::PI);
        let rz = (l + f64::consts::PI).rem_euclid(2.0 * f64::consts::PI);

        (rx.to_degrees(), ry.to_degrees(), rz.to_degrees())
    }
}
