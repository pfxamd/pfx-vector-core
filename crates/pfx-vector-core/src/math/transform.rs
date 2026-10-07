use crate::{Angle, CoreError, CoreResult, Point2, Scalar, Tolerance, Vector2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    pub a: Scalar,
    pub b: Scalar,
    pub c: Scalar,
    pub d: Scalar,
    pub e: Scalar,
    pub f: Scalar,
}

impl Transform2D {
    #[must_use]
    pub const fn new(a: Scalar, b: Scalar, c: Scalar, d: Scalar, e: Scalar, f: Scalar) -> Self {
        Self { a, b, c, d, e, f }
    }

    #[must_use]
    pub const fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    }

    #[must_use]
    pub const fn translation(tx: Scalar, ty: Scalar) -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, tx, ty)
    }

    #[must_use]
    pub const fn scale(sx: Scalar, sy: Scalar) -> Self {
        Self::new(sx, 0.0, 0.0, sy, 0.0, 0.0)
    }

    #[must_use]
    pub fn rotation(angle: Angle) -> Self {
        let (s, c) = angle.as_radians().sin_cos();
        Self::new(c, s, -s, c, 0.0, 0.0)
    }

    #[must_use]
    pub fn skew_x(angle: Angle) -> Self {
        Self::new(1.0, 0.0, angle.as_radians().tan(), 1.0, 0.0, 0.0)
    }

    #[must_use]
    pub fn skew_y(angle: Angle) -> Self {
        Self::new(1.0, angle.as_radians().tan(), 0.0, 1.0, 0.0, 0.0)
    }

    #[must_use]
    pub fn determinant(self) -> Scalar {
        self.a * self.d - self.b * self.c
    }

    #[must_use]
    pub fn then(self, next: Self) -> Self {
        Self::new(
            next.a * self.a + next.c * self.b,
            next.b * self.a + next.d * self.b,
            next.a * self.c + next.c * self.d,
            next.b * self.c + next.d * self.d,
            next.a * self.e + next.c * self.f + next.e,
            next.b * self.e + next.d * self.f + next.f,
        )
    }

    #[must_use]
    pub fn pre_then(self, previous: Self) -> Self {
        previous.then(self)
    }

    pub fn inverse(self, tol: Tolerance) -> CoreResult<Self> {
        let det = self.determinant();
        let scale = self.a.abs().max(self.b.abs()).max(self.c.abs()).max(self.d.abs()).max(1.0);
        if tol.nearly_zero(det, scale * scale) {
            return Err(CoreError::SingularTransform);
        }
        let inv = 1.0 / det;
        Ok(Self::new(
            self.d * inv,
            -self.b * inv,
            -self.c * inv,
            self.a * inv,
            (self.c * self.f - self.d * self.e) * inv,
            (self.b * self.e - self.a * self.f) * inv,
        ))
    }

    #[must_use]
    pub fn transform_point(self, p: Point2) -> Point2 {
        Point2::new(
            self.a * p.x + self.c * p.y + self.e,
            self.b * p.x + self.d * p.y + self.f,
        )
    }

    #[must_use]
    pub fn transform_vector(self, v: Vector2) -> Vector2 {
        Vector2::new(
            self.a * v.x + self.c * v.y,
            self.b * v.x + self.d * v.y,
        )
    }

    #[must_use]
    pub fn is_finite(self) -> bool {
        [self.a, self.b, self.c, self.d, self.e, self.f]
            .into_iter()
            .all(Scalar::is_finite)
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}
