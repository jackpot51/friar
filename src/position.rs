use std::{f64, fmt};

use coordinate::Coordinate;
use reference::Reference;
use perspective::Perspective;
use spheroid::Spheroid;
use vector::Vector;

pub struct Position<'r, R: Reference + 'r> {
    pub reference: &'r R,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl<'r, R: Reference> fmt::Display for Position<'r, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl<'r, R: Reference> Position<'r, R> {
    /// Create a new Position
    pub fn new(reference: &'r R, x: f64, y: f64, z: f64) -> Self {
        Self {
            reference,
            x,
            y,
            z,
        }
    }

    /// Duplicate this position
    pub fn duplicate(&self) -> Self {
        Self::new(self.reference, self.x, self.y, self.z)
    }

    /// Calculate Vector to Position
    pub fn vector(&'r self, to: &Self) -> Vector<'r, R> {
        let x = to.x - self.x;
        let y = to.y - self.y;
        let z = to.z - self.z;

        Vector::new(self.reference, x, y, z)
    }

    /// Convert into Vector from origin
    pub fn to_vector(&'r self) -> Vector<'r, R> {
        Vector::new(self.reference, self.x, self.y, self.z)
    }

    /// Create Perspective from this Position
    pub fn perspective(&'r self, rx: f64, ry: f64, rz: f64) -> Perspective<'r, R> {
        Perspective::new(self, rx, ry, rz)
    }
}

impl<'r, R: Spheroid> Position<'r, R> {
    /// Convert to Coordinate
    ///
    /// Adapted from https://en.wikipedia.org/wiki/Geographic_coordinate_conversion#From_ECEF_to_geodetic_coordinates
    pub fn coordinate(&self) -> Coordinate<'r, R> {
        let a = self.reference.radius_equatorial();
        let asq = a.powi(2);

        let e: f64 = 8.1819190842622e-2; //TODO: Calculate
        let esq = e.powi(2);

        let x = self.x;
        let y = self.y;
        let z = self.z;

        let b = (asq * (1.0 - esq)).sqrt();
        let bsq = b.powi(2);
        let ep = ((asq - bsq)/bsq).sqrt();
        let p = (x.powi(2) + y.powi(2)).sqrt();
        let th = (a * z).atan2(b * p);

        let lon = y.atan2(x).rem_euclid(2.0 * f64::consts::PI);
        let lat = (z + ep.powi(2) * b * th.sin().powi(3)).atan2(p - esq * a * th.cos().powi(3));
        let N = a / (1.0 - esq * lat.sin().powi(2)).sqrt();
        let alt = p / lat.cos() - N;

        self.reference.coordinate(
            lat.to_degrees(),
            lon.to_degrees(),
            alt
        )
    }
}
