use crate::coordinate::Coordinate;
use crate::reference::Reference;

pub trait Spheroid: Reference {
    /// Equatorial radius in meters
    fn radius_equatorial(&self) -> f64;

    /// Polar radius in meters
    fn radius_polar(&self) -> f64;

    /// Create coordinate using this Spheroid as a reference
    fn coordinate<'r>(&'r self, latitude: f64, longitude: f64, elevation: f64) -> Coordinate<'r, Self> {
        Coordinate::new(self, latitude, longitude, elevation)
    }
}
