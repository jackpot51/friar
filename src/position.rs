use std::fmt;

use reference::Reference;
use perspective::Perspective;
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

/* TODO: Find a way to do this without iteration
impl<R: Spheroid> Position<R> {
    /// Convert to Coordinate
    ///
    /// Adapted from https://en.wikipedia.org/wiki/Geographic_coordinate_conversion#From_ECEF_to_geodetic_coordinates
    pub fn coordinate() -> Coordinate<R> {

    }
}
*/
