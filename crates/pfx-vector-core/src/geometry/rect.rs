use crate::{Bounds, CoreError, CoreResult, Point2, Scalar};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: Scalar,
    pub y: Scalar,
    pub width: Scalar,
    pub height: Scalar,
}

impl Rect {
    pub fn new(x: Scalar, y: Scalar, width: Scalar, height: Scalar) -> CoreResult<Self> {
        if ![x, y, width, height].into_iter().all(Scalar::is_finite) {
            return Err(CoreError::InvalidNumber);
        }
        if width < 0.0 || height < 0.0 {
            return Err(CoreError::InvalidGeometry);
        }
        Ok(Self { x, y, width, height })
    }

    #[must_use]
    pub fn bounds(self) -> Bounds {
        Bounds::Finite {
            min: Point2::new(self.x, self.y),
            max: Point2::new(self.x + self.width, self.y + self.height),
        }
    }

    #[must_use]
    pub fn contains(self, point: Point2) -> bool {
        self.bounds().contains(point)
    }
}
