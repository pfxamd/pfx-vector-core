use crate::{Point2, Scalar, Transform2D};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Bounds {
    Empty,
    Finite { min: Point2, max: Point2 },
}

impl Bounds {
    #[must_use]
    pub const fn empty() -> Self {
        Self::Empty
    }

    #[must_use]
    pub fn from_point(p: Point2) -> Self {
        Self::Finite { min: p, max: p }
    }

    #[must_use]
    pub fn from_points(points: &[Point2]) -> Self {
        let mut bounds = Self::Empty;
        for &point in points {
            bounds = bounds.include(point);
        }
        bounds
    }

    #[must_use]
    pub fn include(self, p: Point2) -> Self {
        match self {
            Self::Empty => Self::from_point(p),
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
    pub fn intersection(self, other: Self) -> Self {
        match (self, other) {
            (
                Self::Finite { min: a, max: b },
                Self::Finite { min: c, max: d },
            ) => {
                let min = Point2::new(a.x.max(c.x), a.y.max(c.y));
                let max = Point2::new(b.x.min(d.x), b.y.min(d.y));
                if min.x <= max.x && min.y <= max.y {
                    Self::Finite { min, max }
                } else {
                    Self::Empty
                }
            }
            _ => Self::Empty,
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
    pub fn center(self) -> Option<Point2> {
        match self {
            Self::Empty => None,
            Self::Finite { min, max } => {
                Some(Point2::new((min.x + max.x) * 0.5, (min.y + max.y) * 0.5))
            }
        }
    }

    #[must_use]
    pub fn contains(self, p: Point2) -> bool {
        matches!(
            self,
            Self::Finite { min, max }
                if p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y
        )
    }

    #[must_use]
    pub fn intersects(self, other: Self) -> bool {
        !matches!(self.intersection(other), Self::Empty)
    }

    #[must_use]
    pub fn expanded(self, amount: Scalar) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Finite { min, max } => Self::Finite {
                min: Point2::new(min.x - amount, min.y - amount),
                max: Point2::new(max.x + amount, max.y + amount),
            },
        }
    }

    #[must_use]
    pub fn transformed(self, transform: Transform2D) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Finite { min, max } => Self::from_points(&[
                transform.transform_point(min),
                transform.transform_point(Point2::new(max.x, min.y)),
                transform.transform_point(max),
                transform.transform_point(Point2::new(min.x, max.y)),
            ]),
        }
    }
}
