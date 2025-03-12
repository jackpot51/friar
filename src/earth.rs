use crate::reference::Reference;
use crate::spheroid::Spheroid;

pub struct Earth;

impl Reference for Earth {}

impl Spheroid for Earth {
    /// Equatorial radius of Earth in meters
    /// From https://en.wikipedia.org/wiki/Earth_radius#Equatorial_radius
    fn radius_equatorial(&self) -> f64 {
        6378137.0
    }

    /// Polar radius of Earth in meters
    /// From https://en.wikipedia.org/wiki/Earth_radius#Polar_radius
    fn radius_polar(&self) -> f64 {
        6356752.3
    }
}
