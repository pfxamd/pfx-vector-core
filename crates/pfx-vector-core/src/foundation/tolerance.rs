use super::Scalar;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tolerance {
    pub absolute: Scalar,
    pub relative: Scalar,
    pub angular: Scalar,
    pub flatness: Scalar,
}

impl Tolerance {
    #[must_use]
    pub const fn new(absolute: Scalar, relative: Scalar, angular: Scalar, flatness: Scalar) -> Self {
        Self { absolute, relative, angular, flatness }
    }

    #[inline]
    #[must_use]
    pub fn almost_eq(self, a: Scalar, b: Scalar) -> bool {
        if a == b {
            return true;
        }
        if !a.is_finite() || !b.is_finite() {
            return false;
        }
        let scale = a.abs().max(b.abs()).max(1.0);
        (a - b).abs() <= self.absolute + self.relative * scale
    }
}
