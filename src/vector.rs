use std::fmt;

use crate::reference::Reference;

pub struct Vector<'r, R: Reference + 'r> {
    pub reference: &'r R,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl<'r, R: Reference> fmt::Display for Vector<'r, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl<'r, R: Reference> Vector<'r, R> {
    /// Create a new Vector
    pub fn new(reference: &'r R, x: f64, y: f64, z: f64) -> Self {
        Self {
            reference,
            x,
            y,
            z,
        }
    }

    /// Find the norm (length) of the Vector
    pub fn norm(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    /// Return a normalized Vector (divide by length)
    pub fn normalize(&self) -> Self {
        let l = self.norm();
        self.divide(l)
    }

    /// Dot product with another Vector
    pub fn dot(&self, other: &Vector<'r, R>) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Divide Vector by some factor
    pub fn divide(&self, factor: f64) -> Self {
        Self::new(self.reference, self.x / factor, self.y / factor, self.z / factor)
    }

    /// Multiply Vector by some factor
    pub fn multiply(&self, factor: f64) -> Self {
        Self::new(self.reference, self.x * factor, self.y * factor, self.z * factor)
    }

    /// Cross product with another Vector
    pub fn cross(&self, other: &Vector<'r, R>) -> Self {
        let x = self.y * other.z - self.z * other.y;
        let y = self.z * other.x - self.x * other.z;
        let z = self.x * other.y - self.y * other.x;

        Self::new(self.reference, x, y, z)
    }

    /// Add another vector
    pub fn add(&self, other: &Vector<'r, R>) -> Self {
        Self::new(self.reference, self.x + other.x, self.y + other.y, self.z + other.z)
    }

    /// Subtract another vector
    pub fn subtract(&self, other: &Vector<'r, R>) -> Self {
        Self::new(self.reference, self.x - other.x, self.y - other.y, self.z - other.z)
    }

    /// Find projection of Vector onto another
    pub fn projection(&self, onto: &Vector<'r, R>) -> Self {
        onto.multiply(self.dot(&onto) / onto.dot(&onto))
    }

    /// Rotate Vector around another
    pub fn rotate(&self, other: &Vector<'r, R>, angle: f64) -> Self {
        let k = other.normalize();
        let theta = angle.to_radians();
        self.multiply(theta.cos()).add(
            &k.cross(self).multiply(theta.sin())
        ).add(
            &k.multiply(k.dot(self) * (1.0 - theta.cos()))
        )
    }

    /// Find the heading of the Vector
    pub fn heading(&self) -> f64 {
        (-self.y).asin().to_degrees()
    }

    /// Find the pitch of the Vector
    pub fn pitch(&self) -> f64 {
        self.x.atan2(self.z).to_degrees()
    }
}
