use crate::Scalar;
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Angle(Scalar);
impl Angle {
    #[must_use]
    pub const fn radians(value: Scalar) -> Self {
        Self(value)
    }
    #[must_use]
    pub fn degrees(value: Scalar) -> Self {
        Self(value.to_radians())
    }
    #[must_use]
    pub const fn as_radians(self) -> Scalar {
        self.0
    }
    #[must_use]
    pub fn as_degrees(self) -> Scalar {
        self.0.to_degrees()
    }
    #[must_use]
    pub fn normalized_positive(self) -> Self {
        Self(self.0.rem_euclid(core::f64::consts::TAU))
    }
    #[must_use]
    pub fn normalized_signed(self) -> Self {
        let a = (self.0+core::f64::consts::PI).rem_euclid(core::f64::consts::TAU)-core::f64::consts::PI;
        Self(a)
    }
}
