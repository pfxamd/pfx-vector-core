mod bounds;
mod circle;
mod ellipse;
mod line;
mod rect;

pub use bounds::Bounds;
pub use circle::Circle;
pub use ellipse::Ellipse;
pub use line::LineSegment;
pub use rect::Rect;

use crate::{CoreError, CoreResult, Point2, Scalar};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Polyline {
    pub points: Vec<Point2>,
}

impl Polyline {
    pub fn new(points: Vec<Point2>) -> Self {
        Self { points }
    }

    pub fn bounds(&self) -> Bounds {
        Bounds::from_points(&self.points)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Polygon {
    pub points: Vec<Point2>,
}

impl Polygon {
    pub fn new(points: Vec<Point2>) -> Self {
        Self { points }
    }

    pub fn bounds(&self) -> Bounds {
        Bounds::from_points(&self.points)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoundedRect {
    pub rect: Rect,
    pub radius_x: Scalar,
    pub radius_y: Scalar,
}

impl RoundedRect {
    pub fn new(rect: Rect, radius_x: Scalar, radius_y: Scalar) -> CoreResult<Self> {
        if !radius_x.is_finite() || !radius_y.is_finite() {
            return Err(CoreError::InvalidNumber);
        }
        if radius_x < 0.0 || radius_y < 0.0 {
            return Err(CoreError::InvalidGeometry);
        }
        Ok(Self {
            rect,
            radius_x: radius_x.min(rect.width * 0.5),
            radius_y: radius_y.min(rect.height * 0.5),
        })
    }
}
