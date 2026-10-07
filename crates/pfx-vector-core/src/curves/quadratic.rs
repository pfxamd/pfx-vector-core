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
    #[must_use]
    pub fn derivative_at(self, t: Scalar) -> crate::Vector2 {
        (self.p1 - self.p0) * (2.0 * (1.0 - t)) + (self.p2 - self.p1) * (2.0 * t)
    }

    pub fn tangent_at(self, t: Scalar, tolerance: crate::Tolerance) -> crate::CoreResult<crate::Vector2> {
        self.derivative_at(t).normalized(tolerance)
    }

    #[must_use]
    pub fn split(self, t: Scalar) -> (Self, Self) {
        let a = self.p0.lerp(self.p1, t);
        let b = self.p1.lerp(self.p2, t);
        let middle = a.lerp(b, t);
        (Self::new(self.p0, a, middle), Self::new(middle, b, self.p2))
    }

    #[must_use]
    pub fn bounds(self) -> crate::Bounds {
        let mut bounds = crate::Bounds::from_points(&[self.p0, self.p2]);
        for (a, b, c) in [
            (self.p0.x, self.p1.x, self.p2.x),
            (self.p0.y, self.p1.y, self.p2.y),
        ] {
            let d = a - 2.0 * b + c;
            if d.abs() > f64::EPSILON {
                let t = (a - b) / d;
                if t > 0.0 && t < 1.0 {
                    bounds = bounds.include(self.point_at(t));
                }
            }
        }
        bounds
    }

    #[must_use]
    pub fn reversed(self) -> Self {
        Self::new(self.p2, self.p1, self.p0)
    }
}
