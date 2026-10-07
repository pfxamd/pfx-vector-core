use crate::{
    CoreError, CoreResult, CubicBezier, EllipticalArc, Path, Point2, QuadraticBezier, Segment,
    Tolerance,
};
#[derive(Clone, Debug, PartialEq)]
pub struct FlattenedSubpath {
    pub points: Vec<Point2>,
    pub closed: bool,
}
fn flat_q(q: QuadraticBezier, tol: Tolerance, out: &mut Vec<Point2>, depth: u32) -> CoreResult<()> {
    if depth == 0 {
        return Err(CoreError::IterationLimit);
    }
    if q.flatness() <= tol.flatness {
        out.push(q.p2);
        Ok(())
    } else {
        let (a, b) = q.split(0.5);
        flat_q(a, tol, out, depth - 1)?;
        flat_q(b, tol, out, depth - 1)
    }
}
fn flat_c(c: CubicBezier, tol: Tolerance, out: &mut Vec<Point2>, depth: u32) -> CoreResult<()> {
    let chord = c.p3 - c.p0;
    let len = chord.length();
    let f = if len == 0.0 {
        c.p0.distance_to(c.p1).max(c.p0.distance_to(c.p2))
    } else {
        ((c.p1 - c.p0).cross(chord))
            .abs()
            .max(((c.p2 - c.p0).cross(chord)).abs())
            / len
    };
    if depth == 0 {
        return Err(CoreError::IterationLimit);
    }
    if f <= tol.flatness {
        out.push(c.p3);
        Ok(())
    } else {
        let (a, b) = c.split(0.5);
        flat_c(a, tol, out, depth - 1)?;
        flat_c(b, tol, out, depth - 1)
    }
}
fn flat_a(a: EllipticalArc, tol: Tolerance, out: &mut Vec<Point2>) -> CoreResult<()> {
    let r = a.radius_x.max(a.radius_y).max(tol.flatness);
    let sweep = a.sweep_angle.as_radians().abs();
    let step = (2.0 * (1.0 - (tol.flatness / r).min(1.0)).acos()).max(1e-6);
    let n = (sweep / step).ceil().max(1.0) as usize;
    if n > 1_000_000 {
        return Err(CoreError::IterationLimit);
    }
    for i in 1..=n {
        out.push(a.point_at(i as f64 / n as f64));
    }
    Ok(())
}
pub fn flatten_path(path: &Path, tol: Tolerance) -> CoreResult<Vec<FlattenedSubpath>> {
    let mut result = Vec::new();
    for sub in path.subpaths() {
        let mut pts = vec![sub.start()];
        for &seg in sub.segments() {
            match seg {
                Segment::Line(l) => pts.push(l.end),
                Segment::Quadratic(q) => flat_q(q, tol, &mut pts, 32)?,
                Segment::Cubic(c) => flat_c(c, tol, &mut pts, 32)?,
                Segment::Arc(a) => flat_a(a, tol, &mut pts)?,
            }
        }
        result.push(FlattenedSubpath {
            points: pts,
            closed: sub.is_closed(),
        });
    }
    Ok(result)
}
