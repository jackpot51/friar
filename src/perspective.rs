use position::Position;
use reference::Reference;
use viewport::Viewport;

//TODO: Make this a trait
pub struct Perspective<'r, R: Reference + 'r> {
    position: &'r Position<'r, R>,
    rx: f64,
    ry: f64,
    rz: f64,
}

impl<'r, R: Reference> Reference for Perspective<'r, R> {}

impl<'r, R: Reference> Perspective<'r, R> {
    /// Create a new Perspective
    pub fn new(position: &'r Position<'r, R>, rx: f64, ry: f64, rz: f64) -> Self {
        Self {
            position,
            rx,
            ry,
            rz,
        }
    }

    /// Transform the point into one relative to the Perspective
    ///
    /// Adapted from https://en.wikipedia.org/wiki/3D_projection#Perspective_projection
    pub fn transform(&self, from: &Position<'r, R>) -> Position<Self> {
        let rx = self.rx.to_radians();
        let ry = self.ry.to_radians();
        let rz = self.rz.to_radians();

        let cx = rx.cos();
        let cy = ry.cos();
        let cz = rz.cos();

        let sx = rx.sin();
        let sy = ry.sin();
        let sz = rz.sin();

        let x = from.x - self.position.x;
        let y = from.y - self.position.y;
        let z = from.z - self.position.z;

        let dx = cy * (sz * y + cz * x) - sy * z;
        let dy = sx * (cy * z + sy * (sz * y + cz * x)) + cx * (cz * y - sz * x);
        let dz = cx * (cy * z + sy * (sz * y + cz * x)) - sx * (cz * y - sz * x);

        self.position(dx, dy, dz)
    }

    /// Create a viewport with this perspective
    pub fn viewport(&'r self, x: f64, y: f64, z: f64) -> Viewport<'r, R> {
        Viewport::new(self, x, y, z)
    }
}
