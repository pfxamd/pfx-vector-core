use crate::{
    Bounds, CubicBezier, EllipticalArc, LineSegment, Point2, QuadraticBezier, Scalar, Vector2,
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Segment {
    Line(LineSegment),
    Quadratic(QuadraticBezier),
    Cubic(CubicBezier),
    Arc(EllipticalArc),
}
impl Segment {
    #[must_use]
    pub fn start(self) -> Point2 {
        match self {
            Self::Line(v) => v.start,
            Self::Quadratic(v) => v.p0,
            Self::Cubic(v) => v.p0,
            Self::Arc(v) => v.point_at(0.0),
        }
    }
    #[must_use]
    pub fn end(self) -> Point2 {
        match self {
            Self::Line(v) => v.end,
            Self::Quadratic(v) => v.p2,
            Self::Cubic(v) => v.p3,
            Self::Arc(v) => v.point_at(1.0),
        }
    }
    #[must_use]
    pub fn point_at(self, t: Scalar) -> Point2 {
        match self {
            Self::Line(v) => v.point_at(t),
            Self::Quadratic(v) => v.point_at(t),
            Self::Cubic(v) => v.point_at(t),
            Self::Arc(v) => v.point_at(t),
        }
    }
    #[must_use]
    pub fn derivative_at(self, t: Scalar) -> Vector2 {
        match self {
            Self::Line(v) => v.direction(),
            Self::Quadratic(v) => v.derivative_at(t),
            Self::Cubic(v) => v.derivative_at(t),
            Self::Arc(v) => v.derivative_at(t),
        }
    }
    #[must_use]
    pub fn second_derivative_at(self, t: Scalar) -> Vector2 {
        match self {
            Self::Line(_) => Vector2::new(0.0, 0.0),
            Self::Quadratic(v) => v.second_derivative_at(t),
            Self::Cubic(v) => v.second_derivative_at(t),
            Self::Arc(v) => v.second_derivative_at(t),
        }
    }
    #[must_use]
    pub fn bounds(self) -> Bounds {
        match self {
            Self::Line(v) => v.bounds(),
            Self::Quadratic(v) => v.bounds(),
            Self::Cubic(v) => v.bounds(),
            Self::Arc(v) => v.bounds(),
        }
    }
    #[must_use]
    pub fn reversed(self) -> Self {
        match self {
            Self::Line(v) => Self::Line(v.reversed()),
            Self::Quadratic(v) => Self::Quadratic(v.reversed()),
            Self::Cubic(v) => Self::Cubic(v.reversed()),
            Self::Arc(v) => Self::Arc(v.reversed()),
        }
    }
}
