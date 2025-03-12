use crate::position::Position;
use crate::vector::Vector;

pub trait Reference: Sized {
    /// Create Position using this Reference
    fn position<'r>(&'r self, x: f64, y: f64, z: f64) -> Position<'r, Self> {
        Position::new(self, x, y, z)
    }

    /// Create a Vector using this Reference
    fn vector<'r>(&'r self, x: f64, y: f64, z: f64) -> Vector<'r, Self> {
        Vector::new(self, x, y, z)
    }
}
