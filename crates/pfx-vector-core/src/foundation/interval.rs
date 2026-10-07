use super::{CoreError, CoreResult, Scalar};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    pub min: Scalar,
    pub max: Scalar,
}

impl Interval {
    pub fn new(min: Scalar, max: Scalar) -> CoreResult<Self> {
        if !min.is_finite() || !max.is_finite() {
            return Err(CoreError::InvalidNumber);
        }
        if min > max {
            return Err(CoreError::InvalidGeometry);
        }
        Ok(Self { min, max })
    }

    #[must_use]
    pub fn contains(self, value: Scalar) -> bool {
        value >= self.min && value <= self.max
    }

    #[must_use]
    pub fn length(self) -> Scalar {
        self.max - self.min
    }
}
