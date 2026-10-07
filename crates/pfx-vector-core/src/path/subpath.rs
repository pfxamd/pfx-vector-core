use crate::{Point2, Segment, Tolerance};
#[derive(Clone, Debug, PartialEq)]
pub struct Subpath {
    start: Point2,
    segments: Vec<Segment>,
    closed: bool,
}
impl Subpath {
    pub(crate) fn new(
        start: Point2,
        segments: Vec<Segment>,
        closed: bool,
        tol: Tolerance,
    ) -> Option<Self> {
        let mut current = start;
        for s in &segments {
            if !current.almost_eq(s.start(), tol) {
                return None;
            }
            current = s.end();
        }
        Some(Self {
            start,
            segments,
            closed,
        })
    }
    #[must_use]
    pub const fn start(&self) -> Point2 {
        self.start
    }
    #[must_use]
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        self.closed
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
    #[must_use]
    pub fn end(&self) -> Point2 {
        self.segments.last().map_or(self.start, |s| s.end())
    }
    #[must_use]
    pub fn reversed(&self) -> Self {
        let start = self.end();
        let segs = self.segments.iter().rev().map(|s| s.reversed()).collect();
        Self {
            start,
            segments: segs,
            closed: self.closed,
        }
    }
}
