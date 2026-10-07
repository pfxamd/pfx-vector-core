use crate::{Angle, Bounds, CoreError, CoreResult, Point2, Scalar};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ellipse {
    pub center: Point2,
    pub radius_x: Scalar,
    pub radius_y: Scalar,
    pub rotation: Angle,
}

impl Ellipse {
    pub fn new(center: Point2, radius_x: Scalar, radius_y: Scalar, rotation: Angle) -> CoreResult<Self> {
        if !center.is_finite() || !radius_x.is_finite() || !radius_y.is_finite() {
            return Err(CoreError::InvalidNumber);
        }
        if radius_x < 0.0 || radius_y < 0.0 {
            return Err(CoreError::InvalidGeometry);
        }
        Ok(Self { center, radius_x, radius_y, rotation })
    }
}
