use crate::{Bounds, Point2, Scalar, Vector2};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineSegment {
    pub start: Point2,
    pub end: Point2,
}
impl LineSegment {
    #[must_use]
    pub const fn new(start: Point2, end: Point2) -> Self {
        Self { start, end }
    }
    #[must_use]
    pub fn point_at(self, t: Scalar) -> Point2 {
        self.start.lerp(self.end, t)
    }
    #[must_use]
    pub fn direction(self) -> Vector2 {
        self.end - self.start
    }
    #[must_use]
    pub fn length(self) -> Scalar {
        self.start.distance_to(self.end)
    }
    #[must_use]
    pub fn bounds(self) -> Bounds {
        Bounds::from_points(&[self.start, self.end])
    }
    #[must_use]
    pub fn reversed(self) -> Self {
        Self::new(self.end, self.start)
    }
}
