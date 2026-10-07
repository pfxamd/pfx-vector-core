use crate::{CubicBezier, EllipticalArc, LineSegment, QuadraticBezier};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Segment {
    Line(LineSegment),
    Quadratic(QuadraticBezier),
    Cubic(CubicBezier),
    Arc(EllipticalArc),
}

impl Segment {
    pub fn start(self) -> crate::Point2 {
        match self {
            Self::Line(value) => value.start,
            Self::Quadratic(value) => value.p0,
            Self::Cubic(value) => value.p0,
            Self::Arc(value) => value.point_at(0.0),
        }
    }
}
