use crate::{CubicBezier, EllipticalArc, LineSegment, QuadraticBezier};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Segment {
    Line(LineSegment),
    Quadratic(QuadraticBezier),
    Cubic(CubicBezier),
    Arc(EllipticalArc),
}
