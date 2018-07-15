use position::Position;

pub trait Reference: Sized {
    /// Create Position using this Reference as a reference
    fn position<'r>(&'r self, x: f64, y: f64, z: f64) -> Position<'r, Self> {
        Position::new(self, x, y, z)
    }
}
