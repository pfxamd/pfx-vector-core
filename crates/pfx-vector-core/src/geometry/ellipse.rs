use crate::{Angle, Bounds, CoreError, CoreResult, Point2, Scalar};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ellipse {
    pub center: Point2,
    pub radius_x: Scalar,
    pub radius_y: Scalar,
    pub rotation: Angle,
}
