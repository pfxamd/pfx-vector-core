use crate::{Angle, Bounds, Point2, Scalar, Vector2};

const ANGLE_EPSILON: Scalar = 1.0e-12;

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
        let rotation = self.rotation.as_radians();
        let (sin_rotation, cos_rotation) = rotation.sin_cos();

        let x_extremum =
            (-self.radius_y * sin_rotation).atan2(self.radius_x * cos_rotation);
        let y_extremum =
            (self.radius_y * cos_rotation).atan2(self.radius_x * sin_rotation);

        for angle in [
            x_extremum,
            x_extremum + core::f64::consts::PI,
            y_extremum,
            y_extremum + core::f64::consts::PI,
        ] {
            if self.contains_angle(angle) {
                bounds = bounds.include(self.point_at_angle(angle));
            }
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

    #[must_use]
    pub(crate) fn contains_angle(self, angle: Scalar) -> bool {
        let sweep = self.sweep_angle.as_radians();
        if sweep.abs() >= core::f64::consts::TAU - ANGLE_EPSILON {
            return true;
        }

        if sweep >= 0.0 {
            (angle - self.start_angle.as_radians()).rem_euclid(core::f64::consts::TAU)
                <= sweep + ANGLE_EPSILON
        } else {
            (self.start_angle.as_radians() - angle).rem_euclid(core::f64::consts::TAU)
                <= -sweep + ANGLE_EPSILON
        }
    }

    #[must_use]
    fn point_at_angle(self, angle: Scalar) -> Point2 {
        let (s, c) = angle.sin_cos();
        let (sr, cr) = self.rotation.as_radians().sin_cos();

        Point2::new(
            self.center.x + self.radius_x * c * cr - self.radius_y * s * sr,
            self.center.y + self.radius_x * c * sr + self.radius_y * s * cr,
        )
    }
}
