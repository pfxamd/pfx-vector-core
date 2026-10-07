use crate::{Bounds, CoreError, CoreResult, Path, Point2, Scalar, Tolerance, flatten_path};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StrokeCap {
    Butt,
    Round,
    Square,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StrokeJoin {
    Miter,
    Round,
    Bevel,
}
#[derive(Clone, Debug, PartialEq)]
pub struct StrokeStyle {
    pub width: Scalar,
    pub cap: StrokeCap,
    pub join: StrokeJoin,
    pub miter_limit: Scalar,
    pub dash_array: Vec<Scalar>,
    pub dash_offset: Scalar,
}
impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            width: 1.0,
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
            miter_limit: 4.0,
            dash_array: Vec::new(),
            dash_offset: 0.0,
        }
    }
}
impl StrokeStyle {
    pub fn validate(&self) -> CoreResult<()> {
        if !self.width.is_finite()
            || self.width < 0.0
            || !self.miter_limit.is_finite()
            || self.miter_limit < 0.0
            || !self.dash_offset.is_finite()
            || self.dash_array.iter().any(|v| !v.is_finite() || *v < 0.0)
        {
            return Err(CoreError::InvalidGeometry);
        }
        Ok(())
    }
}
#[must_use]
pub fn normalized_dash_pattern(style: &StrokeStyle) -> Vec<Scalar> {
    if style.dash_array.is_empty() || style.dash_array.iter().all(|v| *v == 0.0) {
        return Vec::new();
    }
    let mut p = style.dash_array.clone();
    if p.len() % 2 == 1 {
        let c = p.clone();
        p.extend(c)
    }
    p
}
pub fn stroke_bounds(path: &Path, style: &StrokeStyle, tol: Tolerance) -> CoreResult<Bounds> {
    style.validate()?;
    let half = style.width * 0.5;
    let mut b = Bounds::Empty;
    for sub in flatten_path(path, tol)? {
        for p in sub.points {
            b = b
                .include(Point2::new(p.x - half, p.y - half))
                .include(Point2::new(p.x + half, p.y + half));
        }
    }
    Ok(b)
}
fn distance_to_segment(p: Point2, a: Point2, b: Point2) -> Scalar {
    let d = b - a;
    let dd = d.dot(d);
    if dd == 0.0 {
        return p.distance_to(a);
    }
    let t = ((p - a).dot(d) / dd).clamp(0.0, 1.0);
    p.distance_to(a + d * t)
}
pub fn stroke_contains_point(
    path: &Path,
    style: &StrokeStyle,
    p: Point2,
    tol: Tolerance,
) -> CoreResult<bool> {
    style.validate()?;
    if style.width == 0.0 {
        return Ok(false);
    }
    let half = style.width * 0.5 + tol.absolute;
    for sub in flatten_path(path, tol)? {
        for w in sub.points.windows(2) {
            if distance_to_segment(p, w[0], w[1]) <= half {
                return Ok(true);
            }
        }
        if sub.closed
            && sub.points.len() > 2
            && distance_to_segment(p, *sub.points.last().unwrap(), sub.points[0]) <= half
        {
            return Ok(true);
        }
    }
    Ok(false)
}
