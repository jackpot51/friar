use std::f64;

use position::Position;
use spheroid::Spheroid;

pub struct Coordinate<'r, R: Spheroid + 'r> {
    pub reference: &'r R,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
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
    pub fn distance(&self, to: Self) -> f64 {
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

    /// Great-circle course to another Coordinate in degrees
    ///
    /// Adapted from https://en.wikipedia.org/wiki/Great-circle_navigation#Course
    pub fn course(&self, to: Self) -> f64 {
        let f1 = self.latitude.to_radians();
        let l1 = self.longitude.to_radians();
        let f2 = to.latitude.to_radians();
        let l2 = to.longitude.to_radians();

        (
            (l2 - l1).sin() * f2.cos()
        ).atan2(
            f1.cos() * f2.sin() - f1.sin() * f2.cos() * (l2 - l1).cos()
        ).mod_euc(2.0 * f64::consts::PI).to_degrees()
    }

    /// Convert to Position
    ///
    /// Adapted from https://en.wikipedia.org/wiki/Geographic_coordinate_conversion#From_geodetic_to_ECEF_coordinates
    pub fn position(&self) -> Position<R> {
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
}
