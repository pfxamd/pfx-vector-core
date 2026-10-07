mod advanced;

use advanced::intersect_curve_pair;
use crate::numeric::{clamp_unit, dedup_sorted, solve_cubic, solve_quadratic};
use crate::{
    CoreResult, Interval, LineSegment, Point2, Scalar, Segment, Tolerance, Vector2,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntersectionKind {
    Crossing,
    Tangent,
    Endpoint,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointIntersection {
    pub point: Point2,
    pub parameter_a: Scalar,
    pub parameter_b: Scalar,
    pub kind: IntersectionKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlapIntersection {
    pub range_a: Interval,
    pub range_b: Interval,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Intersection {
    Point(PointIntersection),
    Overlap(OverlapIntersection),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct IntersectionResult {
    pub intersections: Vec<Intersection>,
}

fn line_line(
    a: LineSegment,
    b: LineSegment,
    tolerance: Tolerance,
) -> CoreResult<IntersectionResult> {
    let r = a.end - a.start;
    let s = b.end - b.start;
    let denominator = r.cross(s);
    let qp = b.start - a.start;

    if tolerance.nearly_zero(denominator, r.length() * s.length()) {
        if !tolerance.nearly_zero(qp.cross(r), qp.length() * r.length()) {
            return Ok(IntersectionResult::default());
        }

        let rr = r.dot(r);
        if rr <= tolerance.absolute {
            return Ok(IntersectionResult::default());
        }

        let mut t0 = qp.dot(r) / rr;
        let mut t1 = (b.end - a.start).dot(r) / rr;

        if t0 > t1 {
            core::mem::swap(&mut t0, &mut t1);
        }

        let low = t0.max(0.0);
        let high = t1.min(1.0);

        if high < low - tolerance.absolute {
            return Ok(IntersectionResult::default());
        }

        if tolerance.almost_eq(low, high) {
            let point = a.point_at(low);
            let ss = s.dot(s);
            let u = if ss == 0.0 {
                0.0
            } else {
                (point - b.start).dot(s) / ss
            };

            return Ok(IntersectionResult {
                intersections: vec![Intersection::Point(PointIntersection {
                    point,
                    parameter_a: low,
                    parameter_b: u,
                    kind: IntersectionKind::Endpoint,
                })],
            });
        }

        let ss = s.dot(s);
        let ub0 = if ss == 0.0 {
            0.0
        } else {
            (a.point_at(low) - b.start).dot(s) / ss
        };
        let ub1 = if ss == 0.0 {
            0.0
        } else {
            (a.point_at(high) - b.start).dot(s) / ss
        };

        return Ok(IntersectionResult {
            intersections: vec![Intersection::Overlap(OverlapIntersection {
                range_a: Interval::new(low, high)?,
                range_b: Interval::new(ub0.min(ub1), ub0.max(ub1))?,
            })],
        });
    }

    let t = qp.cross(s) / denominator;
    let u = qp.cross(r) / denominator;
    let (Some(t), Some(u)) = (clamp_unit(t, tolerance), clamp_unit(u, tolerance)) else {
        return Ok(IntersectionResult::default());
    };

    let kind = if is_endpoint(t) || is_endpoint(u) {
        IntersectionKind::Endpoint
    } else {
        IntersectionKind::Crossing
    };

    Ok(IntersectionResult {
        intersections: vec![Intersection::Point(PointIntersection {
            point: a.point_at(t),
            parameter_a: t,
            parameter_b: u,
            kind,
        })],
    })
}

fn line_curve(
    line: LineSegment,
    segment: Segment,
    tolerance: Tolerance,
) -> CoreResult<IntersectionResult> {
    let direction = line.end - line.start;

    let mut roots = match segment {
        Segment::Quadratic(curve) => {
            let ax = curve.p0.x - 2.0 * curve.p1.x + curve.p2.x;
            let ay = curve.p0.y - 2.0 * curve.p1.y + curve.p2.y;
            let bx = 2.0 * (curve.p1.x - curve.p0.x);
            let by = 2.0 * (curve.p1.y - curve.p0.y);
            let cx = curve.p0.x - line.start.x;
            let cy = curve.p0.y - line.start.y;

            solve_quadratic(
                ax * direction.y - ay * direction.x,
                bx * direction.y - by * direction.x,
                cx * direction.y - cy * direction.x,
            )
        }
        Segment::Cubic(curve) => {
            let ax = -curve.p0.x + 3.0 * curve.p1.x - 3.0 * curve.p2.x + curve.p3.x;
            let ay = -curve.p0.y + 3.0 * curve.p1.y - 3.0 * curve.p2.y + curve.p3.y;
            let bx = 3.0 * curve.p0.x - 6.0 * curve.p1.x + 3.0 * curve.p2.x;
            let by = 3.0 * curve.p0.y - 6.0 * curve.p1.y + 3.0 * curve.p2.y;
            let cx = -3.0 * curve.p0.x + 3.0 * curve.p1.x;
            let cy = -3.0 * curve.p0.y + 3.0 * curve.p1.y;
            let ex = curve.p0.x - line.start.x;
            let ey = curve.p0.y - line.start.y;

            solve_cubic(
                ax * direction.y - ay * direction.x,
                bx * direction.y - by * direction.x,
                cx * direction.y - cy * direction.x,
                ex * direction.y - ey * direction.x,
            )
        }
        Segment::Line(other) => return line_line(line, other, tolerance),
        Segment::Arc(_) => return intersect_curve_pair(Segment::Line(line), segment, tolerance),
    };

    dedup_sorted(&mut roots, tolerance);
    let mut result = IntersectionResult::default();

    for t in roots {
        let Some(t) = clamp_unit(t, tolerance) else {
            continue;
        };

        let point = segment.point_at(t);
        let dd = direction.dot(direction);

        if dd == 0.0 {
            continue;
        }

        let u = (point - line.start).dot(direction) / dd;
        let Some(u) = clamp_unit(u, tolerance) else {
            continue;
        };

        result
            .intersections
            .push(Intersection::Point(PointIntersection {
                point,
                parameter_a: u,
                parameter_b: t,
                kind: classify_line_curve(direction, segment_derivative(segment, t), u, t, tolerance),
            }));
    }

    result.intersections.sort_by(|left, right| {
        let left = point_parameters(left);
        let right = point_parameters(right);
        left.0
            .total_cmp(&right.0)
            .then_with(|| left.1.total_cmp(&right.1))
    });

    Ok(result)
}

fn classify_line_curve(
    line_derivative: Vector2,
    curve_derivative: Vector2,
    line_parameter: Scalar,
    curve_parameter: Scalar,
    tolerance: Tolerance,
) -> IntersectionKind {
    if is_endpoint(line_parameter) || is_endpoint(curve_parameter) {
        return IntersectionKind::Endpoint;
    }

    let scale = line_derivative.length() * curve_derivative.length();
    let angular_tolerance = tolerance.angular.max(1.0e-7);

    if scale == 0.0
        || line_derivative.cross(curve_derivative).abs() <= angular_tolerance * scale
    {
        IntersectionKind::Tangent
    } else {
        IntersectionKind::Crossing
    }
}

fn segment_derivative(segment: Segment, t: Scalar) -> Vector2 {
    match segment {
        Segment::Line(line) => line.direction(),
        Segment::Quadratic(curve) => curve.derivative_at(t),
        Segment::Cubic(curve) => curve.derivative_at(t),
        Segment::Arc(arc) => arc.derivative_at(t),
    }
}

fn is_endpoint(parameter: Scalar) -> bool {
    parameter <= 1.0e-8 || parameter >= 1.0 - 1.0e-8
}

fn point_parameters(intersection: &Intersection) -> (Scalar, Scalar) {
    match intersection {
        Intersection::Point(point) => (point.parameter_a, point.parameter_b),
        Intersection::Overlap(overlap) => (overlap.range_a.min, overlap.range_b.min),
    }
}

pub fn intersect_segments(
    a: Segment,
    b: Segment,
    tolerance: Tolerance,
) -> CoreResult<IntersectionResult> {
    match (a, b) {
        (Segment::Line(line_a), Segment::Line(line_b)) => line_line(line_a, line_b, tolerance),
        (Segment::Line(line), Segment::Quadratic(curve)) => {
            line_curve(line, Segment::Quadratic(curve), tolerance)
        }
        (Segment::Line(line), Segment::Cubic(curve)) => {
            line_curve(line, Segment::Cubic(curve), tolerance)
        }
        (Segment::Quadratic(curve), Segment::Line(line)) => {
            swap_parameters(line_curve(line, Segment::Quadratic(curve), tolerance)?)
        }
        (Segment::Cubic(curve), Segment::Line(line)) => {
            swap_parameters(line_curve(line, Segment::Cubic(curve), tolerance)?)
        }
        _ => intersect_curve_pair(a, b, tolerance),
    }
}

fn swap_parameters(mut result: IntersectionResult) -> CoreResult<IntersectionResult> {
    for intersection in &mut result.intersections {
        match intersection {
            Intersection::Point(point) => {
                core::mem::swap(&mut point.parameter_a, &mut point.parameter_b);
            }
            Intersection::Overlap(overlap) => {
                core::mem::swap(&mut overlap.range_a, &mut overlap.range_b);
            }
        }
    }

    Ok(result)
}
