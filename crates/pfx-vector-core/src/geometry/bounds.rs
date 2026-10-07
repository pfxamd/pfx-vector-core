use crate::{Point2, Scalar};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Bounds {
    #[default]
    Empty,
    Finite {
        min: Point2,
        max: Point2,
    },
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

    #[must_use]
    pub fn intersects(self, other: Self) -> bool {
        match (self, other) {
            (
                Self::Finite {
                    min: left_min,
                    max: left_max,
                },
                Self::Finite {
                    min: right_min,
                    max: right_max,
                },
            ) => {
                left_min.x <= right_max.x
                    && left_max.x >= right_min.x
                    && left_min.y <= right_max.y
                    && left_max.y >= right_min.y
            }
            _ => false,
        }
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
    pub fn distance_squared_to_point(self, point: Point2) -> Scalar {
        match self {
            Self::Empty => Scalar::INFINITY,
            Self::Finite { min, max } => {
                let dx = if point.x < min.x {
                    min.x - point.x
                } else if point.x > max.x {
                    point.x - max.x
                } else {
                    0.0
                };
                let dy = if point.y < min.y {
                    min.y - point.y
                } else if point.y > max.y {
                    point.y - max.y
                } else {
                    0.0
                };

                dx * dx + dy * dy
            }
        }
    }
}
