use std::cmp;

use position::Position;
use reference::Reference;
use viewport::Viewport;

//TODO: Turn into trait
pub struct Screen<'r, R: Reference + 'r> {
    pub viewport: &'r Viewport<'r, R>,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl<'r, R: Reference> Screen<'r, R> {
    /// Create new Viewport
    pub fn new(viewport: &'r Viewport<'r, R>, x: f64, y: f64, z: f64) -> Self {
        Self {
            viewport,
            x,
            y,
            z,
        }
    }

    /// Transform from reference Position into the Viewport's plane
    //TODO: Return Position<Viewport>
    pub fn transform(&self, point: &Position<'r, R>) -> (f64, f64, f64) {
        let (bx, by, bz) = self.viewport.transform(point);

        let a = self.x.max(self.y);
        let ax = a/self.x;
        let ay = a/self.y;

        let sx = (bx * ax + 1.0)/2.0 * self.x;
        let sy = (by * ay + 1.0)/2.0 * self.y;
        let sz = bz * self.z;

        (sx, sy, sz)
    }
}
