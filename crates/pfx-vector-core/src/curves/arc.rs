use crate::{Angle, Bounds, Point2, Scalar, Vector2};

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
    #[must_use]
    pub const fn new(
        center: Point2,
        rx: Scalar,
        ry: Scalar,
        rotation: Angle,
        start: Angle,
        sweep: Angle,
    ) -> Self {
        Self {
            center,
            radius_x: rx,
            radius_y: ry,
            rotation,
            start_angle: start,
            sweep_angle: sweep,
        }
    }

    #[must_use]
    pub fn point_at(self, t: Scalar) -> Point2 {
        let theta = self.start_angle.as_radians() + self.sweep_angle.as_radians() * t;
        let (s, c) = theta.sin_cos();
        let (sr, cr) = self.rotation.as_radians().sin_cos();

        Point2::new(
            self.center.x + self.radius_x * c * cr - self.radius_y * s * sr,
            self.center.y + self.radius_x * c * sr + self.radius_y * s * cr,
        )
    }

    #[must_use]
    pub fn derivative_at(self, t: Scalar) -> Vector2 {
        let theta = self.start_angle.as_radians() + self.sweep_angle.as_radians() * t;
        let (s, c) = theta.sin_cos();
        let (sr, cr) = self.rotation.as_radians().sin_cos();
        let w = self.sweep_angle.as_radians();

        Vector2::new(
            (-self.radius_x * s * cr - self.radius_y * c * sr) * w,
            (-self.radius_x * s * sr + self.radius_y * c * cr) * w,
        )
    }

    #[must_use]
    pub fn bounds(self) -> Bounds {
        let mut bounds = Bounds::from_points(&[self.point_at(0.0), self.point_at(1.0)]);

        for index in 0..360 {
            let t = f64::from(index) / 359.0;
            bounds = bounds.include(self.point_at(t));
        }

        bounds
    }

    #[must_use]
    pub fn reversed(self) -> Self {
        Self::new(
            self.center,
            self.radius_x,
            self.radius_y,
            self.rotation,
            Angle::radians(self.start_angle.as_radians() + self.sweep_angle.as_radians()),
            Angle::radians(-self.sweep_angle.as_radians()),
        )
    }
}
