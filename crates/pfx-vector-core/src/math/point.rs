use core::ops::{
    Add, Sub
};
use crate::{
    Scalar, Tolerance, Vector2
};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point2 {
    pub x: Scalar, pub y: Scalar
}
impl Point2 {
    #[must_use]
    pub const fn new(x: Scalar, y: Scalar) -> Self {
        Self {
            x, y
        }
    }
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
    #[must_use]
    pub fn distance_squared_to(self, other: Self) -> Scalar {
        (self - other).length_squared()
    }
    #[must_use]
    pub fn distance_to(self, other: Self) -> Scalar {
        self.distance_squared_to(other).sqrt()
    }
    #[must_use]
    pub fn lerp(self, other: Self, t: Scalar) -> Self {
        Self::new(self.x + (other.x-self.x)*t, self.y + (other.y-self.y)*t)
    }
    #[must_use]
    pub fn almost_eq(self, other: Self, tol: Tolerance) -> bool {
        tol.almost_eq(self.x, other.x) && tol.almost_eq(self.y, other.y)
    }
}
impl Add<Vector2> for Point2 {
    type Output = Self;
    fn add(self, rhs: Vector2) -> Self {
        Self::new(self.x+rhs.x, self.y+rhs.y)
    }
}
impl Sub<Vector2> for Point2 {
    type Output = Self;
    fn sub(self, rhs: Vector2) -> Self {
        Self::new(self.x-rhs.x, self.y-rhs.y)
    }
}
impl Sub<Point2> for Point2 {
    type Output = Vector2;
    fn sub(self, rhs: Point2) -> Vector2 {
        Vector2::new(self.x-rhs.x, self.y-rhs.y)
    }
}
