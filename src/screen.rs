use crate::position::Position;
use crate::reference::Reference;
use crate::viewport::Viewport;

//TODO: Turn into trait
pub struct Screen<'r, R: Reference + 'r> {
    viewport: &'r Viewport<'r, R>,
    pub x: f64, //TODO: make private, due to cached calculations
    pub y: f64, //TODO: make private, due to cached calculations
    theta: f64,
    // Cached calculations
    ct: f64,
    st: f64,
    a: f64,
    ax: f64,
    ay: f64,
}

impl<'r, R: Reference> Screen<'r, R> {
    /// Create new Screen
    pub fn new(viewport: &'r Viewport<'r, R>, x: f64, y: f64, theta: f64) -> Self {
        let radt = theta.to_radians();
        let ct = radt.cos();
        let st = radt.sin();

        let a = x.max(y);
        let ax = a/x;
        let ay = a/y;

        Self {
            viewport,
            x,
            y,
            theta,
            ct,
            st,
            a,
            ax,
            ay,
        }
    }

    /// Transform from reference Position into the Screen's plane
    pub fn transform(&self, point: &Position<'r, R>) -> (f64, f64, f64) {
        let (bx, by, bz) = self.viewport.transform(point);

        let x = bx * self.ct - by * self.st;
        let y = by * self.ct + bx * self.st;

        let sx = (x * self.ax + 1.0)/2.0 * self.x;
        let sy = (y * self.ay + 1.0)/2.0 * self.y;
        let sz = bz * self.a;

        (sx, sy, sz)
    }
}
