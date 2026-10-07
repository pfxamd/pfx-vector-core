use crate::{Bounds, CoreError, CoreResult, Point2, Scalar};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle {
    pub center: Point2,
    pub radius: Scalar,
}
impl Circle {
    pub fn new(center: Point2, radius: Scalar) -> CoreResult<Self> {
        if !center.is_finite() || !radius.is_finite() {
            return Err(CoreError::InvalidNumber);
        }
        if radius < 0.0 {
            return Err(CoreError::InvalidGeometry);
        }
        Ok(Self { center, radius })
    }
    #[must_use]
    pub fn bounds(self) -> Bounds {
        Bounds::from_points(&[
            Point2::new(self.center.x - self.radius, self.center.y - self.radius),
            Point2::new(self.center.x + self.radius, self.center.y + self.radius),
        ])
    }
}
