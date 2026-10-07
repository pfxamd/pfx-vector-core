use core::ops::{Add, Div, Mul, Neg, Sub};
use crate::{CoreError, CoreResult, Scalar, Tolerance};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2 { pub x: Scalar, pub y: Scalar }
impl Vector2 {
    #[must_use] pub const fn new(x: Scalar, y: Scalar) -> Self { Self { x, y } }
    #[must_use] pub fn length_squared(self) -> Scalar { self.dot(self) }
    #[must_use] pub fn length(self) -> Scalar { self.length_squared().sqrt() }
    #[must_use] pub fn dot(self, other: Self) -> Scalar { self.x*other.x+self.y*other.y }
    #[must_use] pub fn cross(self, other: Self) -> Scalar { self.x*other.y-self.y*other.x }
    #[must_use] pub fn perpendicular(self) -> Self { Self::new(-self.y, self.x) }
    pub fn normalized(self, tol: Tolerance) -> CoreResult<Self> {
        let len = self.length();
        if tol.nearly_zero(len, self.x.abs().max(self.y.abs())) { return Err(CoreError::DegenerateOperation); }
        Ok(self/len)
    }
}
impl Add for Vector2 { type Output=Self; fn add(self,rhs:Self)->Self{Self::new(self.x+rhs.x,self.y+rhs.y)} }
impl Sub for Vector2 { type Output=Self; fn sub(self,rhs:Self)->Self{Self::new(self.x-rhs.x,self.y-rhs.y)} }
impl Mul<Scalar> for Vector2 { type Output=Self; fn mul(self,rhs:Scalar)->Self{Self::new(self.x*rhs,self.y*rhs)} }
impl Div<Scalar> for Vector2 { type Output=Self; fn div(self,rhs:Scalar)->Self{Self::new(self.x/rhs,self.y/rhs)} }
impl Neg for Vector2 { type Output=Self; fn neg(self)->Self{Self::new(-self.x,-self.y)} }
