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
        tolerance: Tolerance,
    ) -> Option<Self> {
        let mut current = start;
        for segment in &segments {
            if !current.almost_eq(segment.start(), tolerance) {
                return None;
            }
            current = segment.end();
        }
        Some(Self { start, segments, closed })
    }

    pub fn start(&self) -> Point2 { self.start }
    pub fn segments(&self) -> &[Segment] { &self.segments }
    pub fn is_closed(&self) -> bool { self.closed }
    pub fn is_empty(&self) -> bool { self.segments.is_empty() }
    pub fn end(&self) -> Point2 {
        self.segments.last().map_or(self.start, |segment| segment.end())
    }
}
