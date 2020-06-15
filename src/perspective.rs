use position::Position;
use reference::Reference;
use viewport::Viewport;

//TODO: Make this a trait
pub struct Perspective<'r, R: Reference + 'r> {
    position: &'r Position<'r, R>,
    rx: f64,
    ry: f64,
    rz: f64,
    // Cached calculations
    cx: f64,
    cy: f64,
    cz: f64,
    sx: f64,
    sy: f64,
    sz: f64,
}

impl<'r, R: Reference> Reference for Perspective<'r, R> {}

impl<'r, R: Reference> Perspective<'r, R> {
    /// Create a new Perspective
    pub fn new(position: &'r Position<'r, R>, rx: f64, ry: f64, rz: f64) -> Self {
        let radx = rx.to_radians();
        let rady = ry.to_radians();
        let radz = rz.to_radians();

        let cx = radx.cos();
        let cy = rady.cos();
        let cz = radz.cos();

        let sx = radx.sin();
        let sy = rady.sin();
        let sz = radz.sin();

        Self {
            position,
            rx,
            ry,
            rz,
            cx,
            cy,
            cz,
            sx,
            sy,
            sz,
        }
    }

    /// Transform the point into one relative to the Perspective
    ///
    /// Adapted from https://en.wikipedia.org/wiki/3D_projection#Perspective_projection
    pub fn transform(&self, from: &Position<'r, R>) -> Position<Self> {
        let x = from.x - self.position.x;
        let y = from.y - self.position.y;
        let z = from.z - self.position.z;

        let cy_z = self.cy * z;
        let cz_x = self.cz * x;
        let cz_y = self.cz * y;
        let sy_z = self.sy * z;
        let sz_x = self.sz * x;
        let sz_y = self.sz * y;

        let cz_y_m_sz_x = cz_y - sz_x;
        let sz_y_p_cz_x = sz_y + cz_x;

        // Name would otherwise be incomprehensible
        let boom = cy_z + self.sy * sz_y_p_cz_x;

        let dx = self.cy * sz_y_p_cz_x - sy_z;
        let dy = self.sx * boom + self.cx * cz_y_m_sz_x;
        let dz = self.cx * boom - self.sx * cz_y_m_sz_x;

        self.position(dx, dy, dz)
    }

    /// Create a viewport with this perspective
    pub fn viewport(&'r self, x: f64, y: f64, z: f64) -> Viewport<'r, R> {
        Viewport::new(self, x, y, z)
    }
}
