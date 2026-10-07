use crate::{
    CoreError, CoreResult, CubicBezier, EllipticalArc, LineSegment, Path, Point2, QuadraticBezier,
    Segment, Subpath, Tolerance
};
#[derive(Debug)]
pub struct PathBuilder {
    tol: Tolerance, subpaths: Vec<Subpath>, current_start: Option<Point2>, current: Option<Point2>,
    segments: Vec<Segment>
}
impl Default for PathBuilder {
    fn default() -> Self {
        Self::new()
    }
}
impl PathBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::with_tolerance(Tolerance::default())
    }
    #[must_use]
    pub fn with_tolerance(tol: Tolerance) -> Self {
        Self {
            tol, subpaths: Vec::new(), current_start: None, current: None, segments: Vec::new()
        }
    }
    fn flush(&mut self, closed: bool) -> CoreResult<()> {
        if let Some(start) = self.current_start.take() {
            let segs = core::mem::take(&mut self.segments);
            let sub = Subpath::new(start, segs, closed, self.tol).ok_or(CoreError::InvalidGeometry)?;
            self.current = Some(if closed {
                start
            } else {
                sub.end()
            });
            self.subpaths.push(sub);
        }
        Ok(())
    }
    pub fn move_to(&mut self, p: Point2) -> CoreResult<&mut Self> {
        if !p.is_finite() {
            return Err(CoreError::InvalidNumber)
        }
        self.flush(false)?;
        self.current_start = Some(p);
        self.current = Some(p);
        Ok(self)
    }
    pub fn line_to(&mut self, p: Point2) -> CoreResult<&mut Self> {
        let c = self.current.ok_or(CoreError::InvalidGeometry)?;
        if !p.is_finite() {
            return Err(CoreError::InvalidNumber)
        }
        self.segments.push(Segment::Line(LineSegment::new(c, p)));
        self.current = Some(p);
        Ok(self)
    }
    pub fn quad_to(&mut self, c1: Point2, p: Point2) -> CoreResult<&mut Self> {
        let c = self.current.ok_or(CoreError::InvalidGeometry)?;
        if !c1.is_finite() || !p.is_finite() {
            return Err(CoreError::InvalidNumber)
        }
        self.segments.push(Segment::Quadratic(QuadraticBezier::new(c, c1, p)));
        self.current = Some(p);
        Ok(self)
    }
    pub fn cubic_to(&mut self, c1: Point2, c2: Point2, p: Point2) -> CoreResult<&mut Self> {
        let c = self.current.ok_or(CoreError::InvalidGeometry)?;
        if !c1.is_finite() || !c2.is_finite() || !p.is_finite() {
            return Err(CoreError::InvalidNumber)
        }
        self.segments.push(Segment::Cubic(CubicBezier::new(c, c1, c2, p)));
        self.current = Some(p);
        Ok(self)
    }
    pub fn arc_to(&mut self, arc: EllipticalArc) -> CoreResult<&mut Self> {
        let c = self.current.ok_or(CoreError::InvalidGeometry)?;
        if !c.almost_eq(arc.point_at(0.0), self.tol) {
            return Err(CoreError::InvalidGeometry)
        }
        self.current = Some(arc.point_at(1.0));
        self.segments.push(Segment::Arc(arc));
        Ok(self)
    }
    pub fn close(&mut self) -> CoreResult<&mut Self> {
        if self.current_start.is_none() {
            return Err(CoreError::InvalidGeometry)
        }
        self.flush(true)?;
        self.current = None;
        Ok(self)
    }
    pub fn finish(mut self) -> CoreResult<Path> {
        self.flush(false)?;
        Ok(Path::from_subpaths(self.subpaths))
    }
}
