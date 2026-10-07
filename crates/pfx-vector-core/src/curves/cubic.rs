use crate::{Point2, Scalar};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    pub p0: Point2,
    pub p1: Point2,
    pub p2: Point2,
    pub p3: Point2,
}

impl CubicBezier {
    #[must_use]
    pub const fn new(p0: Point2, p1: Point2, p2: Point2, p3: Point2) -> Self {
        Self { p0, p1, p2, p3 }
    }

    #[must_use]
    pub fn point_at(self, t: Scalar) -> Point2 {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let t2 = t * t;
        Point2::new(
            self.p0.x * mt2 * mt + 3.0 * self.p1.x * mt2 * t + 3.0 * self.p2.x * mt * t2 + self.p3.x * t2 * t,
            self.p0.y * mt2 * mt + 3.0 * self.p1.y * mt2 * t + 3.0 * self.p2.y * mt * t2 + self.p3.y * t2 * t,
        )
    }
}
