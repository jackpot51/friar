pub struct Triangle<'r, R: Reference> {
    pub a: Position<'r, R>,
    pub b: Position<'r, R>,
    pub c: Position<'r, R>,
}

impl<'r, R: Reference> Triangle<'r, R> {
    /// Create a new Triangle from three Positions
    pub fn new(a: Position<'r, R>, b: Position<'r, R>, c: Position<'r, R>) {
        Triangle {
            a,
            b,
            c
        }
    }
}
