use crate::{Point2, Tolerance};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Orientation {
    Clockwise,
    CounterClockwise,
    Collinear,
}
#[must_use]
pub fn orientation(a: Point2, b: Point2, c: Point2, tol: Tolerance) -> Orientation {
    let ab = b - a;
    let ac = c - a;
    let cross = ab.cross(ac);
    let scale = ab.length() * ac.length();
    if tol.nearly_zero(cross, scale) {
        Orientation::Collinear
    } else if cross > 0.0 {
        Orientation::CounterClockwise
    } else {
        Orientation::Clockwise
    }
}
