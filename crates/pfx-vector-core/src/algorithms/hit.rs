use crate::{
    CoreResult, FillRule, Path, Point2, StrokeStyle, Tolerance, contains_point,
    stroke_contains_point,
};
pub fn hit_fill(path: &Path, p: Point2, rule: FillRule, tol: Tolerance) -> CoreResult<bool> {
    contains_point(path, p, rule, tol)
}
pub fn hit_stroke(path: &Path, p: Point2, style: &StrokeStyle, tol: Tolerance) -> CoreResult<bool> {
    stroke_contains_point(path, style, p, tol)
}
