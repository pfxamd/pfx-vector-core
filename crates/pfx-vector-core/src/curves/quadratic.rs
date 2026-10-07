use crate::{Point2, Scalar};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuadraticBezier {
    pub p0: Point2,
    pub p1: Point2,
    pub p2: Point2,
}

impl QuadraticBezier {
    #[must_use]
    pub const fn new(p0: Point2, p1: Point2, p2: Point2) -> Self {
        Self { p0, p1, p2 }
    }

    #[must_use]
    pub fn point_at(self, t: Scalar) -> Point2 {
        let mt = 1.0 - t;
        Point2::new(
            mt * mt * self.p0.x + 2.0 * mt * t * self.p1.x + t * t * self.p2.x,
            mt * mt * self.p0.y + 2.0 * mt * t * self.p1.y + t * t * self.p2.y,
        )
    }
}
