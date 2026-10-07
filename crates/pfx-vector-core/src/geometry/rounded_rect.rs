use crate::{Rect, Scalar};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoundedRect {
    pub rect: Rect,
    pub radius_x: Scalar,
    pub radius_y: Scalar,
}
