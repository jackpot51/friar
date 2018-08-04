use perspective::Perspective;
use position::Position;
use reference::Reference;
use screen::Screen;

//TODO: Turn into trait
pub struct Viewport<'r, R: Reference + 'r> {
    pub perspective: &'r Perspective<'r, R>,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl<'r, R: Reference> Viewport<'r, R> {
    /// Create new Viewport
    pub fn new(perspective: &'r Perspective<'r, R>, x: f64, y: f64, z: f64) -> Self {
        Self {
            perspective,
            x,
            y,
            z,
        }
    }

    /// Transform from reference Position into the Viewport's plane
    //TODO: Return Position<Viewport>?
    pub fn transform(&self, point: &Position<'r, R>) -> (f64, f64, f64) {
        let d = self.perspective.transform(point);

        let bz = self.z / d.z;
        let bx = bz * d.x + self.x;
        let by = bz * d.y + self.y;

        (bx, by, bz)
    }

    pub fn screen(&'r self, x: f64, y: f64, theta: f64) -> Screen<'r, R> {
        Screen::new(self, x, y, theta)
    }
}
