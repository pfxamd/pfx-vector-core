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
}
