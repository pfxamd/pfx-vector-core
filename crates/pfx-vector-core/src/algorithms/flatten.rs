use crate::{
    CoreError, CoreResult, CubicBezier, EllipticalArc, Path, Point2, QuadraticBezier, Scalar,
    Segment, Tolerance,
};

#[derive(Clone, Debug, PartialEq)]
pub struct FlattenedSubpath {
    pub points: Vec<Point2>,
    pub closed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FlattenedSegmentEdge {
    pub start: Point2,
    pub end: Point2,
    pub t_start: Scalar,
    pub t_end: Scalar,
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

fn flat_q_edges(
    q: QuadraticBezier,
    tol: Tolerance,
    out: &mut Vec<FlattenedSegmentEdge>,
    depth: u32,
    t_start: Scalar,
    t_end: Scalar,
) -> CoreResult<()> {
    if depth == 0 {
        return Err(CoreError::IterationLimit);
    }
    if q.flatness() <= tol.flatness {
        out.push(FlattenedSegmentEdge {
            start: q.p0,
            end: q.p2,
            t_start,
            t_end,
        });
        Ok(())
    } else {
        let (a, b) = q.split(0.5);
        let t_mid = (t_start + t_end) * 0.5;
        flat_q_edges(a, tol, out, depth - 1, t_start, t_mid)?;
        flat_q_edges(b, tol, out, depth - 1, t_mid, t_end)
    }
}

fn flat_c_edges(
    c: CubicBezier,
    tol: Tolerance,
    out: &mut Vec<FlattenedSegmentEdge>,
    depth: u32,
    t_start: Scalar,
    t_end: Scalar,
) -> CoreResult<()> {
    let chord = c.p3 - c.p0;
    let len = chord.length();
    let flatness = if len == 0.0 {
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
    if flatness <= tol.flatness {
        out.push(FlattenedSegmentEdge {
            start: c.p0,
            end: c.p3,
            t_start,
            t_end,
        });
        Ok(())
    } else {
        let (a, b) = c.split(0.5);
        let t_mid = (t_start + t_end) * 0.5;
        flat_c_edges(a, tol, out, depth - 1, t_start, t_mid)?;
        flat_c_edges(b, tol, out, depth - 1, t_mid, t_end)
    }
}

pub(crate) fn flatten_segment_edges(
    segment: Segment,
    tol: Tolerance,
) -> CoreResult<Vec<FlattenedSegmentEdge>> {
    let mut edges = Vec::new();

    match segment {
        Segment::Line(line) => edges.push(FlattenedSegmentEdge {
            start: line.start,
            end: line.end,
            t_start: 0.0,
            t_end: 1.0,
        }),
        Segment::Quadratic(curve) => {
            flat_q_edges(curve, tol, &mut edges, 32, 0.0, 1.0)?;
        }
        Segment::Cubic(curve) => {
            flat_c_edges(curve, tol, &mut edges, 32, 0.0, 1.0)?;
        }
        Segment::Arc(arc) => {
            let r = arc.radius_x.max(arc.radius_y).max(tol.flatness);
            let sweep = arc.sweep_angle.as_radians().abs();
            let step = (2.0 * (1.0 - (tol.flatness / r).min(1.0)).acos()).max(1e-6);
            let count = (sweep / step).ceil().max(1.0) as usize;
            if count > 1_000_000 {
                return Err(CoreError::IterationLimit);
            }

            for index in 0..count {
                let t_start = index as Scalar / count as Scalar;
                let t_end = (index + 1) as Scalar / count as Scalar;
                edges.push(FlattenedSegmentEdge {
                    start: arc.point_at(t_start),
                    end: arc.point_at(t_end),
                    t_start,
                    t_end,
                });
            }
        }
    }

    Ok(edges)
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
