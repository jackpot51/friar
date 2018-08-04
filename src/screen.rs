use position::Position;
use reference::Reference;
use viewport::Viewport;

//TODO: Turn into trait
pub struct Screen<'r, R: Reference + 'r> {
    pub viewport: &'r Viewport<'r, R>,
    pub x: f64,
    pub y: f64,
    pub theta: f64,
}

impl<'r, R: Reference> Screen<'r, R> {
    /// Create new Screen
    pub fn new(viewport: &'r Viewport<'r, R>, x: f64, y: f64, theta: f64) -> Self {
        Self {
            viewport,
            x,
            y,
            theta,
        }
    }

    /// Transform from reference Position into the Screen's plane
    pub fn transform(&self, point: &Position<'r, R>) -> (f64, f64, f64) {
        let (bx, by, bz) = self.viewport.transform(point);

        let t = self.theta.to_radians();
        let ct = t.cos();
        let st = t.sin();

        let x = bx * ct - by * st;
        let y = by * ct + bx * st;

        let a = self.x.max(self.y);
        let ax = a/self.x;
        let ay = a/self.y;

        let sx = (x * ax + 1.0)/2.0 * self.x;
        let sy = (y * ay + 1.0)/2.0 * self.y;
        let sz = bz * a;

        (sx, sy, sz)
    }
}
