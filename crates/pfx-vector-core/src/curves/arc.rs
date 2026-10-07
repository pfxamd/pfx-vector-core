use crate::{Angle, CoreError, CoreResult, Point2, Scalar};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EllipticalArc {
    pub center: Point2,
    pub radius_x: Scalar,
    pub radius_y: Scalar,
    pub rotation: Angle,
    pub start_angle: Angle,
    pub sweep_angle: Angle,
}

impl EllipticalArc {
    pub fn new(
        center: Point2,
        radius_x: Scalar,
        radius_y: Scalar,
        rotation: Angle,
        start_angle: Angle,
        sweep_angle: Angle,
    ) -> CoreResult<Self> {
        if !center.is_finite() || !radius_x.is_finite() || !radius_y.is_finite() {
            return Err(CoreError::InvalidNumber);
        }
        if radius_x < 0.0 || radius_y < 0.0 {
            return Err(CoreError::InvalidGeometry);
        }
        Ok(Self { center, radius_x, radius_y, rotation, start_angle, sweep_angle })
    }

    #[must_use]
    pub fn angle_at(self, t: Scalar) -> Scalar {
        self.start_angle.as_radians() + self.sweep_angle.as_radians() * t
    }

    #[must_use]
    pub fn point_at(self, t: Scalar) -> Point2 {
        let angle = self.angle_at(t);
        let (sa, ca) = angle.sin_cos();
        let (sr, cr) = self.rotation.as_radians().sin_cos();
        Point2::new(
            self.center.x + self.radius_x * ca * cr - self.radius_y * sa * sr,
            self.center.y + self.radius_x * ca * sr + self.radius_y * sa * cr,
        )
    }
    #[must_use]
    pub fn derivative_at(self, t: Scalar) -> crate::Vector2 {
        let angle = self.angle_at(t);
        let (sa, ca) = angle.sin_cos();
        let (sr, cr) = self.rotation.as_radians().sin_cos();
        let sweep = self.sweep_angle.as_radians();
        crate::Vector2::new(
            (-self.radius_x * sa * cr - self.radius_y * ca * sr) * sweep,
            (-self.radius_x * sa * sr + self.radius_y * ca * cr) * sweep,
        )
    }
    pub fn tangent_at(
        self,
        t: Scalar,
        tolerance: crate::Tolerance,
    ) -> CoreResult<crate::Vector2> {
        self.derivative_at(t).normalized(tolerance)
    }

    #[must_use]
    pub fn split(self, t: Scalar) -> (Self, Self) {
        let t = t.clamp(0.0, 1.0);
        let sweep = self.sweep_angle.as_radians();
        let middle = self.start_angle.as_radians() + sweep * t;
        (
            Self { sweep_angle: Angle::radians(sweep * t), ..self },
            Self {
                start_angle: Angle::radians(middle),
                sweep_angle: Angle::radians(sweep * (1.0 - t)),
                ..self
            },
        )
    }
    #[must_use]
    pub fn reversed(self) -> Self {
        Self {
            start_angle: Angle::radians(
                self.start_angle.as_radians() + self.sweep_angle.as_radians(),
            ),
            sweep_angle: Angle::radians(-self.sweep_angle.as_radians()),
            ..self
        }
    }
}
