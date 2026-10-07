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
    #[must_use]
    pub fn derivative_at(self, t: Scalar) -> crate::Vector2 {
        let mt = 1.0 - t;
        (self.p1 - self.p0) * (3.0 * mt * mt)
            + (self.p2 - self.p1) * (6.0 * mt * t)
            + (self.p3 - self.p2) * (3.0 * t * t)
    }

    pub fn tangent_at(self, t: Scalar, tolerance: crate::Tolerance) -> crate::CoreResult<crate::Vector2> {
        self.derivative_at(t).normalized(tolerance)
    }

    #[must_use]
    pub fn split(self, t: Scalar) -> (Self, Self) {
        let a = self.p0.lerp(self.p1, t);
        let b = self.p1.lerp(self.p2, t);
        let c = self.p2.lerp(self.p3, t);
        let d = a.lerp(b, t);
        let e = b.lerp(c, t);
        let middle = d.lerp(e, t);
        (
            Self::new(self.p0, a, d, middle),
            Self::new(middle, e, c, self.p3),
        )
    }

    #[must_use]
    pub fn bounds(self) -> crate::Bounds {
        let mut bounds = crate::Bounds::from_points(&[self.p0, self.p3]);
        for values in [
            (self.p0.x, self.p1.x, self.p2.x, self.p3.x),
            (self.p0.y, self.p1.y, self.p2.y, self.p3.y),
        ] {
            let (p0, p1, p2, p3) = values;
            let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
            let b = 2.0 * (p0 - 2.0 * p1 + p2);
            let c = p1 - p0;
            for t in crate::numeric::solve_quadratic(3.0 * a, b, c) {
                if t > 0.0 && t < 1.0 {
                    bounds = bounds.include(self.point_at(t));
                }
            }
        }
        bounds
    }

    #[must_use]
    pub fn reversed(self) -> Self {
        Self::new(self.p3, self.p2, self.p1, self.p0)
    }
}
