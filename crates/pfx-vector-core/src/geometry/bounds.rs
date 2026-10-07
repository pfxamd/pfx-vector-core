use crate::{Point2, Scalar};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Bounds {
    Empty,
    Finite { min: Point2, max: Point2 },
}
impl Default for Bounds {
    fn default() -> Self {
        Self::Empty
    }
}
impl Bounds {
    #[must_use]
    pub fn from_points(points: &[Point2]) -> Self {
        let mut b = Self::Empty;
        for &p in points {
            b = b.include(p);
        }
        b
    }
    #[must_use]
    pub fn include(self, p: Point2) -> Self {
        match self {
            Self::Empty => Self::Finite { min: p, max: p },
            Self::Finite { min, max } => Self::Finite {
                min: Point2::new(min.x.min(p.x), min.y.min(p.y)),
                max: Point2::new(max.x.max(p.x), max.y.max(p.y)),
            },
        }
    }
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        match other {
            Self::Empty => self,
            Self::Finite { min, max } => self.include(min).include(max),
        }
    }
    #[must_use]
    pub fn width(self) -> Scalar {
        match self {
            Self::Empty => 0.0,
            Self::Finite { min, max } => max.x - min.x,
        }
    }
    #[must_use]
    pub fn height(self) -> Scalar {
        match self {
            Self::Empty => 0.0,
            Self::Finite { min, max } => max.y - min.y,
        }
    }
    #[must_use]
    pub fn contains(self, p: Point2) -> bool {
        match self {
            Self::Empty => false,
            Self::Finite { min, max } => {
                p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y
            }
        }
    }
}
