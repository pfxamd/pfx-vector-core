use super::{
    Intersection, IntersectionKind, IntersectionResult, OverlapIntersection, PointIntersection,
};
use crate::{
    Angle, Bounds, CoreError, CoreResult, CubicBezier, EllipticalArc, Interval, LineSegment,
    Point2, QuadraticBezier, Scalar, Segment, Tolerance, Vector2,
};

const MAX_SEARCH_DEPTH: u32 = 36;
const MAX_SEARCH_NODES: usize = 200_000;
const PARAMETER_EPSILON: Scalar = 1.0e-8;

#[derive(Clone, Copy)]
struct SearchNode {
    a0: Scalar,
    a1: Scalar,
    b0: Scalar,
    b1: Scalar,
    depth: u32,
}

pub(super) fn intersect_curve_pair(
    a: Segment,
    b: Segment,
    tolerance: Tolerance,
) -> CoreResult<IntersectionResult> {
    if let Some(overlap) = detect_overlap(a, b, tolerance)? {
        return Ok(IntersectionResult {
            intersections: vec![overlap],
        });
    }

    let mut result = IntersectionResult::default();
    let mut stack = vec![SearchNode {
        a0: 0.0,
        a1: 1.0,
        b0: 0.0,
        b1: 1.0,
        depth: 0,
    }];
    let mut visited = 0usize;

    while let Some(node) = stack.pop() {
        visited += 1;
        if visited > MAX_SEARCH_NODES {
            return Err(CoreError::IterationLimit);
        }

        let piece_a = subsegment(a, node.a0, node.a1);
        let piece_b = subsegment(b, node.b0, node.b1);
        let bounds_a = piece_a.bounds();
        let bounds_b = piece_b.bounds();
        let spatial_tolerance = spatial_tolerance(bounds_a, bounds_b, tolerance);

        if !bounds_overlap(bounds_a, bounds_b, spatial_tolerance) {
            continue;
        }

        let extent_a = bounds_extent(bounds_a);
        let extent_b = bounds_extent(bounds_b);
        let parameter_small =
            (node.a1 - node.a0) <= PARAMETER_EPSILON && (node.b1 - node.b0) <= PARAMETER_EPSILON;
        let spatial_small =
            extent_a <= spatial_tolerance * 4.0 && extent_b <= spatial_tolerance * 4.0;

        if node.depth >= MAX_SEARCH_DEPTH || parameter_small || spatial_small {
            if let Some(hit) = candidate_from_box(a, b, node, tolerance, spatial_tolerance) {
                push_unique(&mut result, hit, tolerance, spatial_tolerance);
            }
            continue;
        }

        if extent_a >= extent_b {
            let midpoint = (node.a0 + node.a1) * 0.5;
            stack.push(SearchNode {
                a0: midpoint,
                a1: node.a1,
                depth: node.depth + 1,
                ..node
            });
            stack.push(SearchNode {
                a0: node.a0,
                a1: midpoint,
                depth: node.depth + 1,
                ..node
            });
        } else {
            let midpoint = (node.b0 + node.b1) * 0.5;
            stack.push(SearchNode {
                b0: midpoint,
                b1: node.b1,
                depth: node.depth + 1,
                ..node
            });
            stack.push(SearchNode {
                b0: node.b0,
                b1: midpoint,
                depth: node.depth + 1,
                ..node
            });
        }
    }

    result.intersections.sort_by(|left, right| {
        parameter_key(left)
            .0
            .total_cmp(&parameter_key(right).0)
            .then_with(|| parameter_key(left).1.total_cmp(&parameter_key(right).1))
    });

    Ok(result)
}

fn candidate_from_box(
    a: Segment,
    b: Segment,
    node: SearchNode,
    tolerance: Tolerance,
    spatial_tolerance: Scalar,
) -> Option<Intersection> {
    let a_samples = [node.a0, (node.a0 + node.a1) * 0.5, node.a1];
    let b_samples = [node.b0, (node.b0 + node.b1) * 0.5, node.b1];

    let mut best = None;

    for parameter_a in a_samples {
        for parameter_b in b_samples {
            let distance_squared = a
                .point_at(parameter_a)
                .distance_squared_to(b.point_at(parameter_b));

            if best.is_none_or(|(best_distance, _, _)| distance_squared < best_distance) {
                best = Some((distance_squared, parameter_a, parameter_b));
            }
        }
    }

    let (_, initial_a, initial_b) = best?;
    let (mut parameter_a, mut parameter_b) =
        refine_parameters(a, b, initial_a, initial_b, node, tolerance);

    parameter_a = canonical_parameter(a, parameter_a, tolerance);
    parameter_b = canonical_parameter(b, parameter_b, tolerance);

    let point_a = a.point_at(parameter_a);
    let point_b = b.point_at(parameter_b);
    let distance = point_a.distance_to(point_b);
    let residual_tolerance = intersection_residual_tolerance(point_a, point_b, tolerance);

    if distance > residual_tolerance {
        return None;
    }

    let point = Point2::new((point_a.x + point_b.x) * 0.5, (point_a.y + point_b.y) * 0.5);

    Some(Intersection::Point(PointIntersection {
        point,
        parameter_a,
        parameter_b,
        kind: classify_kind(a, b, parameter_a, parameter_b, tolerance),
    }))
}

fn refine_parameters(
    a: Segment,
    b: Segment,
    mut parameter_a: Scalar,
    mut parameter_b: Scalar,
    node: SearchNode,
    tolerance: Tolerance,
) -> (Scalar, Scalar) {
    for _ in 0..12 {
        let point_a = a.point_at(parameter_a);
        let point_b = b.point_at(parameter_b);
        let residual = point_a - point_b;
        let derivative_a = derivative(a, parameter_a);
        let derivative_b = derivative(b, parameter_b);
        let denominator = derivative_a.cross(derivative_b);
        let scale = derivative_a.length() * derivative_b.length();

        if tolerance.nearly_zero(denominator, scale) {
            break;
        }

        let delta_a = -residual.cross(derivative_b) / denominator;
        let delta_b = derivative_a.cross(residual) / denominator;

        if !delta_a.is_finite() || !delta_b.is_finite() {
            break;
        }

        let next_a = (parameter_a + delta_a).clamp(node.a0, node.a1);
        let next_b = (parameter_b + delta_b).clamp(node.b0, node.b1);

        if (next_a - parameter_a).abs() <= PARAMETER_EPSILON * 0.1
            && (next_b - parameter_b).abs() <= PARAMETER_EPSILON * 0.1
        {
            parameter_a = next_a;
            parameter_b = next_b;
            break;
        }

        parameter_a = next_a;
        parameter_b = next_b;
    }

    (parameter_a, parameter_b)
}

fn classify_kind(
    a: Segment,
    b: Segment,
    parameter_a: Scalar,
    parameter_b: Scalar,
    tolerance: Tolerance,
) -> IntersectionKind {
    if is_endpoint(parameter_a) || is_endpoint(parameter_b) {
        return IntersectionKind::Endpoint;
    }

    let derivative_a = derivative(a, parameter_a);
    let derivative_b = derivative(b, parameter_b);
    let scale = derivative_a.length() * derivative_b.length();
    let angular_tolerance = tolerance.angular.max(1.0e-7);

    if scale == 0.0 || derivative_a.cross(derivative_b).abs() <= angular_tolerance * scale {
        IntersectionKind::Tangent
    } else {
        IntersectionKind::Crossing
    }
}

fn derivative(segment: Segment, t: Scalar) -> Vector2 {
    match segment {
        Segment::Line(line) => line.direction(),
        Segment::Quadratic(curve) => curve.derivative_at(t),
        Segment::Cubic(curve) => curve.derivative_at(t),
        Segment::Arc(arc) => arc.derivative_at(t),
    }
}

fn subsegment(segment: Segment, t0: Scalar, t1: Scalar) -> Segment {
    match segment {
        Segment::Line(line) => {
            Segment::Line(LineSegment::new(line.point_at(t0), line.point_at(t1)))
        }
        Segment::Quadratic(curve) => Segment::Quadratic(curve.subcurve(t0, t1)),
        Segment::Cubic(curve) => Segment::Cubic(cubic_subcurve(curve, t0, t1)),
        Segment::Arc(arc) => {
            let start = arc.start_angle.as_radians() + arc.sweep_angle.as_radians() * t0;
            let sweep = arc.sweep_angle.as_radians() * (t1 - t0);
            Segment::Arc(EllipticalArc::new(
                arc.center,
                arc.radius_x,
                arc.radius_y,
                arc.rotation,
                Angle::radians(start),
                Angle::radians(sweep),
            ))
        }
    }
}

fn cubic_subcurve(curve: CubicBezier, t0: Scalar, t1: Scalar) -> CubicBezier {
    if t0 <= 0.0 && t1 >= 1.0 {
        return curve;
    }

    let (_, right) = curve.split(t0.clamp(0.0, 1.0));
    let local = if t0 >= 1.0 {
        0.0
    } else {
        ((t1 - t0) / (1.0 - t0)).clamp(0.0, 1.0)
    };

    right.split(local).0
}

fn bounds_overlap(a: Bounds, b: Bounds, epsilon: Scalar) -> bool {
    match (a, b) {
        (
            Bounds::Finite {
                min: min_a,
                max: max_a,
            },
            Bounds::Finite {
                min: min_b,
                max: max_b,
            },
        ) => {
            max_a.x + epsilon >= min_b.x
                && max_b.x + epsilon >= min_a.x
                && max_a.y + epsilon >= min_b.y
                && max_b.y + epsilon >= min_a.y
        }
        _ => false,
    }
}

fn bounds_extent(bounds: Bounds) -> Scalar {
    bounds.width().max(bounds.height())
}

fn spatial_tolerance(a: Bounds, b: Bounds, tolerance: Tolerance) -> Scalar {
    let scale = bounds_coordinate_scale(a)
        .max(bounds_coordinate_scale(b))
        .max(1.0);
    (tolerance.absolute + tolerance.relative * scale)
        .max(tolerance.flatness * 1.0e-3)
        .max(1.0e-10)
}

fn bounds_coordinate_scale(bounds: Bounds) -> Scalar {
    match bounds {
        Bounds::Empty => 1.0,
        Bounds::Finite { min, max } => min
            .x
            .abs()
            .max(min.y.abs())
            .max(max.x.abs())
            .max(max.y.abs()),
    }
}

fn intersection_residual_tolerance(
    point_a: Point2,
    point_b: Point2,
    tolerance: Tolerance,
) -> Scalar {
    let scale = point_a
        .x
        .abs()
        .max(point_a.y.abs())
        .max(point_b.x.abs())
        .max(point_b.y.abs())
        .max(1.0);

    tolerance.absolute + tolerance.relative * scale
}

fn push_unique(
    result: &mut IntersectionResult,
    candidate: Intersection,
    tolerance: Tolerance,
    spatial_tolerance: Scalar,
) {
    let Intersection::Point(candidate_point) = candidate else {
        result.intersections.push(candidate);
        return;
    };

    let duplicate = result.intersections.iter().any(|existing| {
        let Intersection::Point(existing_point) = existing else {
            return false;
        };

        let point_distance = existing_point.point.distance_to(candidate_point.point);
        let same_parameterized_event = point_distance <= spatial_tolerance * 4.0
            && (existing_point.parameter_a - candidate_point.parameter_a).abs()
                <= PARAMETER_EPSILON * 8.0
            && (existing_point.parameter_b - candidate_point.parameter_b).abs()
                <= PARAMETER_EPSILON * 8.0;
        let tangent_merge_distance = spatial_tolerance.max(tolerance.flatness * 0.25);
        let same_tangent_event = existing_point.kind == IntersectionKind::Tangent
            && candidate_point.kind == IntersectionKind::Tangent
            && point_distance <= tangent_merge_distance;

        same_parameterized_event || same_tangent_event
    });

    if !duplicate {
        result
            .intersections
            .push(Intersection::Point(PointIntersection {
                point: candidate_point.point,
                parameter_a: snap_parameter(candidate_point.parameter_a, tolerance),
                parameter_b: snap_parameter(candidate_point.parameter_b, tolerance),
                kind: candidate_point.kind,
            }));
    }
}

fn canonical_parameter(segment: Segment, parameter: Scalar, tolerance: Tolerance) -> Scalar {
    let parameter = snap_parameter(parameter, tolerance);

    if parameter == 1.0 && segment.start().almost_eq(segment.end(), tolerance) {
        0.0
    } else {
        parameter
    }
}

fn snap_parameter(parameter: Scalar, tolerance: Tolerance) -> Scalar {
    let epsilon = PARAMETER_EPSILON.max(tolerance.absolute.sqrt().min(1.0e-5));

    if parameter.abs() <= epsilon {
        0.0
    } else if (1.0 - parameter).abs() <= epsilon {
        1.0
    } else {
        parameter.clamp(0.0, 1.0)
    }
}

fn is_endpoint(parameter: Scalar) -> bool {
    parameter <= PARAMETER_EPSILON || parameter >= 1.0 - PARAMETER_EPSILON
}

fn parameter_key(intersection: &Intersection) -> (Scalar, Scalar) {
    match intersection {
        Intersection::Point(point) => (point.parameter_a, point.parameter_b),
        Intersection::Overlap(overlap) => (overlap.range_a.min, overlap.range_b.min),
    }
}

fn detect_overlap(
    a: Segment,
    b: Segment,
    tolerance: Tolerance,
) -> CoreResult<Option<Intersection>> {
    match (a, b) {
        (Segment::Quadratic(left), Segment::Quadratic(right)) => {
            if same_quadratic(left, right, tolerance)
                || same_quadratic(left, right.reversed(), tolerance)
            {
                return Ok(Some(full_overlap()?));
            }
        }
        (Segment::Cubic(left), Segment::Cubic(right)) => {
            if same_cubic(left, right, tolerance) || same_cubic(left, right.reversed(), tolerance) {
                return Ok(Some(full_overlap()?));
            }
        }
        (Segment::Arc(left), Segment::Arc(right)) => {
            return arc_overlap(left, right, tolerance);
        }
        _ => {}
    }

    Ok(None)
}

fn same_quadratic(left: QuadraticBezier, right: QuadraticBezier, tolerance: Tolerance) -> bool {
    left.p0.almost_eq(right.p0, tolerance)
        && left.p1.almost_eq(right.p1, tolerance)
        && left.p2.almost_eq(right.p2, tolerance)
}

fn same_cubic(left: CubicBezier, right: CubicBezier, tolerance: Tolerance) -> bool {
    left.p0.almost_eq(right.p0, tolerance)
        && left.p1.almost_eq(right.p1, tolerance)
        && left.p2.almost_eq(right.p2, tolerance)
        && left.p3.almost_eq(right.p3, tolerance)
}

fn full_overlap() -> CoreResult<Intersection> {
    Ok(Intersection::Overlap(OverlapIntersection {
        range_a: Interval::new(0.0, 1.0)?,
        range_b: Interval::new(0.0, 1.0)?,
    }))
}

fn arc_overlap(
    left: EllipticalArc,
    right: EllipticalArc,
    tolerance: Tolerance,
) -> CoreResult<Option<Intersection>> {
    if !left.center.almost_eq(right.center, tolerance)
        || !tolerance.almost_eq(left.radius_x, right.radius_x)
        || !tolerance.almost_eq(left.radius_y, right.radius_y)
        || angle_distance(left.rotation.as_radians(), right.rotation.as_radians())
            > tolerance.angular.max(1.0e-10)
    {
        return Ok(None);
    }

    let left_sweep = left.sweep_angle.as_radians();
    let right_sweep = right.sweep_angle.as_radians();

    if left_sweep.abs() <= tolerance.angular || right_sweep.abs() <= tolerance.angular {
        return Ok(None);
    }

    let left_start = left.start_angle.as_radians();
    let left_end = left_start + left_sweep;
    let left_low = left_start.min(left_end);
    let left_high = left_start.max(left_end);

    let mut best: Option<(Scalar, Scalar, Scalar)> = None;

    for turn in -2..=2 {
        let shifted_start =
            right.start_angle.as_radians() + Scalar::from(turn) * core::f64::consts::TAU;
        let shifted_end = shifted_start + right_sweep;
        let right_low = shifted_start.min(shifted_end);
        let right_high = shifted_start.max(shifted_end);
        let low = left_low.max(right_low);
        let high = left_high.min(right_high);
        let length = high - low;

        if length > tolerance.angular.max(1.0e-10)
            && best.is_none_or(|(best_length, _, _)| length > best_length)
        {
            best = Some((length, low, high));
        }
    }

    let Some((_, low, high)) = best else {
        return Ok(None);
    };

    let left_t0 = ((low - left_start) / left_sweep).clamp(0.0, 1.0);
    let left_t1 = ((high - left_start) / left_sweep).clamp(0.0, 1.0);

    let mut right_parameters = None;
    for turn in -2..=2 {
        let shifted_start =
            right.start_angle.as_radians() + Scalar::from(turn) * core::f64::consts::TAU;
        let t0 = (low - shifted_start) / right_sweep;
        let t1 = (high - shifted_start) / right_sweep;

        if (-PARAMETER_EPSILON..=1.0 + PARAMETER_EPSILON).contains(&t0)
            && (-PARAMETER_EPSILON..=1.0 + PARAMETER_EPSILON).contains(&t1)
        {
            right_parameters = Some((t0.clamp(0.0, 1.0), t1.clamp(0.0, 1.0)));
            break;
        }
    }

    let Some((right_t0, right_t1)) = right_parameters else {
        return Ok(None);
    };

    Ok(Some(Intersection::Overlap(OverlapIntersection {
        range_a: Interval::new(left_t0.min(left_t1), left_t0.max(left_t1))?,
        range_b: Interval::new(right_t0.min(right_t1), right_t0.max(right_t1))?,
    })))
}

fn angle_distance(left: Scalar, right: Scalar) -> Scalar {
    let delta = (left - right).rem_euclid(core::f64::consts::TAU);
    delta.min(core::f64::consts::TAU - delta)
}
