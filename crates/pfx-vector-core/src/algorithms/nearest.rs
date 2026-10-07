use super::measure::segment_length_to_t;
use crate::{
    CoreError, CoreResult, CubicBezier, EllipticalArc, LineSegment, Path, PathLocation, Point2,
    QuadraticBezier, Scalar, Segment, Tolerance, Vector2, segment_length,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegmentClosestPoint {
    pub point: Point2,
    pub distance: Scalar,
    pub distance_squared: Scalar,
    pub t: Scalar,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClosestPointResult {
    pub point: Point2,
    pub distance: Scalar,
    pub distance_squared: Scalar,
    pub location: PathLocation,
}

pub fn closest_point_on_segment(
    segment: Segment,
    point: Point2,
    tolerance: Tolerance,
) -> CoreResult<SegmentClosestPoint> {
    if !point.is_finite() {
        return Err(CoreError::InvalidNumber);
    }
    validate_segment(segment)?;

    let mut parameters = vec![0.0, 1.0];

    match segment {
        Segment::Line(line) => return Ok(closest_line(line, point)),
        Segment::Quadratic(curve) => {
            parameters.extend(quadratic_stationary_parameters(curve, point));
        }
        Segment::Cubic(curve) => {
            parameters.extend(cubic_stationary_parameters(curve, point));
        }
        Segment::Arc(arc) => {
            parameters.extend(arc_stationary_parameters(arc, point));
        }
    }

    parameters.retain(|t| t.is_finite() && *t >= -1.0e-12 && *t <= 1.0 + 1.0e-12);
    for parameter in &mut parameters {
        *parameter = parameter.clamp(0.0, 1.0);
    }
    parameters.sort_by(Scalar::total_cmp);
    parameters.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-11);

    let mut best = None;
    for t in parameters {
        let candidate = segment.point_at(t);
        if !candidate.is_finite() {
            return Err(CoreError::InvalidNumber);
        }
        let distance_squared = candidate.distance_squared_to(point);
        if !distance_squared.is_finite() {
            return Err(CoreError::InvalidNumber);
        }

        let value = SegmentClosestPoint {
            point: candidate,
            distance: distance_squared.sqrt(),
            distance_squared,
            t,
        };
        if best.is_none_or(|current: SegmentClosestPoint| {
            distance_squared < current.distance_squared
                || (distance_squared == current.distance_squared && t < current.t)
        }) {
            best = Some(value);
        }
    }

    let result = best.ok_or(CoreError::DegenerateOperation)?;
    let numerical = tolerance
        .absolute
        .max(tolerance.relative * result.point.x.abs().max(result.point.y.abs()).max(1.0));
    if result.distance <= numerical {
        return Ok(SegmentClosestPoint {
            distance: 0.0,
            distance_squared: 0.0,
            ..result
        });
    }

    Ok(result)
}

pub fn closest_point(
    path: &Path,
    point: Point2,
    tolerance: Tolerance,
) -> CoreResult<ClosestPointResult> {
    if !point.is_finite() {
        return Err(CoreError::InvalidNumber);
    }

    let mut best: Option<(SegmentClosestPoint, usize, usize, Scalar)> = None;
    let mut accumulated = 0.0;

    for (subpath_index, subpath) in path.subpaths().iter().enumerate() {
        for (segment_index, &segment) in subpath.segments().iter().enumerate() {
            let candidate = closest_point_on_segment(segment, point, tolerance)?;
            let start_distance = accumulated;

            if best.is_none_or(|(current, current_subpath, current_segment, _)| {
                candidate.distance_squared < current.distance_squared
                    || (candidate.distance_squared == current.distance_squared
                        && (subpath_index, segment_index, candidate.t)
                            < (current_subpath, current_segment, current.t))
            }) {
                best = Some((candidate, subpath_index, segment_index, start_distance));
            }

            accumulated += segment_length(segment, tolerance)?;
        }
    }

    let (candidate, subpath_index, segment_index, start_distance) =
        best.ok_or(CoreError::DegenerateOperation)?;
    let segment = path.subpaths()[subpath_index].segments()[segment_index];
    let distance = start_distance + segment_length_to_t(segment, candidate.t, tolerance)?;

    Ok(ClosestPointResult {
        point: candidate.point,
        distance: candidate.distance,
        distance_squared: candidate.distance_squared,
        location: PathLocation {
            subpath_index,
            segment_index,
            t: candidate.t,
            distance: distance.min(accumulated),
        },
    })
}

fn closest_line(line: LineSegment, point: Point2) -> SegmentClosestPoint {
    let direction = line.end - line.start;
    let length_squared = direction.length_squared();
    let t = if length_squared == 0.0 {
        0.0
    } else {
        ((point - line.start).dot(direction) / length_squared).clamp(0.0, 1.0)
    };
    let candidate = line.point_at(t);
    let distance_squared = candidate.distance_squared_to(point);

    SegmentClosestPoint {
        point: candidate,
        distance: distance_squared.sqrt(),
        distance_squared,
        t,
    }
}

fn quadratic_stationary_parameters(curve: QuadraticBezier, point: Point2) -> Vec<Scalar> {
    let a = vector_from_points(Point2::new(
        curve.p0.x - 2.0 * curve.p1.x + curve.p2.x,
        curve.p0.y - 2.0 * curve.p1.y + curve.p2.y,
    ));
    let b = (curve.p1 - curve.p0) * 2.0;
    let c = curve.p0 - point;

    polynomial_roots_in_unit_interval(&[
        c.dot(b),
        b.dot(b) + 2.0 * c.dot(a),
        3.0 * a.dot(b),
        2.0 * a.dot(a),
    ])
}

fn cubic_stationary_parameters(curve: CubicBezier, point: Point2) -> Vec<Scalar> {
    let x = cubic_power_coefficients(curve.p0.x, curve.p1.x, curve.p2.x, curve.p3.x, point.x);
    let y = cubic_power_coefficients(curve.p0.y, curve.p1.y, curve.p2.y, curve.p3.y, point.y);
    let dx = [x[1], 2.0 * x[2], 3.0 * x[3]];
    let dy = [y[1], 2.0 * y[2], 3.0 * y[3]];

    let mut stationary = [0.0; 6];
    add_polynomial_product(&mut stationary, &x, &dx);
    add_polynomial_product(&mut stationary, &y, &dy);
    polynomial_roots_in_unit_interval(&stationary)
}

fn cubic_power_coefficients(
    p0: Scalar,
    p1: Scalar,
    p2: Scalar,
    p3: Scalar,
    query: Scalar,
) -> [Scalar; 4] {
    [
        p0 - query,
        3.0 * (p1 - p0),
        3.0 * (p0 - 2.0 * p1 + p2),
        -p0 + 3.0 * p1 - 3.0 * p2 + p3,
    ]
}

fn add_polynomial_product<const N: usize>(
    output: &mut [Scalar; N],
    left: &[Scalar],
    right: &[Scalar],
) {
    for (left_index, &left_value) in left.iter().enumerate() {
        for (right_index, &right_value) in right.iter().enumerate() {
            output[left_index + right_index] += left_value * right_value;
        }
    }
}

fn arc_stationary_parameters(arc: EllipticalArc, point: Point2) -> Vec<Scalar> {
    let delta = point - arc.center;
    let rotation = arc.rotation.as_radians();
    let (sin_rotation, cos_rotation) = rotation.sin_cos();
    let query_x = delta.x * cos_rotation + delta.y * sin_rotation;
    let query_y = -delta.x * sin_rotation + delta.y * cos_rotation;

    let a = arc.radius_y * arc.radius_y - arc.radius_x * arc.radius_x;
    let b = arc.radius_x * query_x;
    let c = arc.radius_y * query_y;
    let polynomial = [-c, 2.0 * (a + b), 0.0, 2.0 * (b - a), c];

    let start = arc.start_angle.as_radians();
    let sweep = arc.sweep_angle.as_radians();
    if sweep == 0.0 {
        return Vec::new();
    }

    let lower = start.min(start + sweep);
    let upper = start.max(start + sweep);
    let mut angles = Vec::new();

    for root in polynomial_real_roots(&polynomial) {
        let base = 2.0 * root.atan();
        append_periodic_angle(&mut angles, base, lower, upper);
    }

    let first_pi = ((lower - core::f64::consts::PI) / core::f64::consts::TAU).ceil() as i64;
    let last_pi = ((upper - core::f64::consts::PI) / core::f64::consts::TAU).floor() as i64;
    for turn in first_pi..=last_pi {
        let theta = core::f64::consts::PI + turn as Scalar * core::f64::consts::TAU;
        if arc_stationary_value(arc, query_x, query_y, theta).abs()
            <= stationary_epsilon(arc.radius_x.max(arc.radius_y))
        {
            angles.push(theta);
        }
    }

    let mut parameters = angles
        .into_iter()
        .map(|theta| (theta - start) / sweep)
        .filter(|t| *t >= -1.0e-12 && *t <= 1.0 + 1.0e-12)
        .collect::<Vec<_>>();
    parameters.sort_by(Scalar::total_cmp);
    parameters.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-11);
    parameters
}

fn append_periodic_angle(angles: &mut Vec<Scalar>, base: Scalar, lower: Scalar, upper: Scalar) {
    let first = ((lower - base) / core::f64::consts::TAU).ceil() as i64;
    let last = ((upper - base) / core::f64::consts::TAU).floor() as i64;
    for turn in first..=last {
        angles.push(base + turn as Scalar * core::f64::consts::TAU);
    }
}

fn arc_stationary_value(
    arc: EllipticalArc,
    query_x: Scalar,
    query_y: Scalar,
    theta: Scalar,
) -> Scalar {
    let (sin_theta, cos_theta) = theta.sin_cos();
    let x = arc.radius_x * cos_theta;
    let y = arc.radius_y * sin_theta;
    (x - query_x) * (-arc.radius_x * sin_theta) + (y - query_y) * (arc.radius_y * cos_theta)
}

fn stationary_epsilon(scale: Scalar) -> Scalar {
    scale.abs().max(1.0).powi(2) * 1.0e-11
}

fn polynomial_roots_in_unit_interval(coefficients: &[Scalar]) -> Vec<Scalar> {
    polynomial_real_roots(coefficients)
        .into_iter()
        .filter(|root| *root >= -1.0e-12 && *root <= 1.0 + 1.0e-12)
        .map(|root| root.clamp(0.0, 1.0))
        .collect()
}

fn polynomial_real_roots(coefficients: &[Scalar]) -> Vec<Scalar> {
    let Some(normalized) = normalize_polynomial(coefficients) else {
        return Vec::new();
    };
    real_roots_normalized(&normalized)
}

fn normalize_polynomial(coefficients: &[Scalar]) -> Option<Vec<Scalar>> {
    if coefficients.iter().any(|value| !value.is_finite()) {
        return None;
    }

    let scale = coefficients
        .iter()
        .copied()
        .map(Scalar::abs)
        .fold(0.0, Scalar::max);
    if scale == 0.0 {
        return None;
    }

    let mut normalized = coefficients
        .iter()
        .map(|value| *value / scale)
        .collect::<Vec<_>>();
    while normalized.len() > 1
        && normalized
            .last()
            .is_some_and(|value| value.abs() <= 1.0e-14)
    {
        normalized.pop();
    }

    (normalized.len() > 1).then_some(normalized)
}

fn real_roots_normalized(coefficients: &[Scalar]) -> Vec<Scalar> {
    let degree = coefficients.len() - 1;
    if degree == 1 {
        return vec![-coefficients[0] / coefficients[1]];
    }

    let derivative = coefficients
        .iter()
        .enumerate()
        .skip(1)
        .map(|(power, coefficient)| *coefficient * power as Scalar)
        .collect::<Vec<_>>();
    let critical = normalize_polynomial(&derivative)
        .map_or_else(Vec::new, |values| real_roots_normalized(&values));

    let leading = coefficients[degree].abs();
    let bound = 1.0
        + coefficients[..degree]
            .iter()
            .map(|value| value.abs() / leading)
            .fold(0.0, Scalar::max);

    let mut breakpoints = vec![-bound];
    breakpoints.extend(
        critical
            .iter()
            .copied()
            .filter(|root| root.is_finite() && *root > -bound && *root < bound),
    );
    breakpoints.push(bound);
    breakpoints.sort_by(Scalar::total_cmp);
    breakpoints.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-12);

    let mut roots = Vec::new();
    for &critical_root in &critical {
        if critical_root >= -bound
            && critical_root <= bound
            && evaluate_polynomial(coefficients, critical_root).abs() <= 1.0e-10
        {
            roots.push(critical_root);
        }
    }

    for interval in breakpoints.windows(2) {
        let left = interval[0];
        let right = interval[1];
        let left_value = evaluate_polynomial(coefficients, left);
        let right_value = evaluate_polynomial(coefficients, right);

        if left_value.abs() <= 1.0e-12 {
            roots.push(left);
        }
        if right_value.abs() <= 1.0e-12 {
            roots.push(right);
        }
        if left_value.signum() == right_value.signum() {
            continue;
        }

        let mut low = left;
        let mut high = right;
        let mut low_value = left_value;
        for _ in 0..80 {
            let middle = (low + high) * 0.5;
            let value = evaluate_polynomial(coefficients, middle);
            if value.abs() <= 1.0e-14 {
                low = middle;
                high = middle;
                break;
            }
            if low_value.signum() == value.signum() {
                low = middle;
                low_value = value;
            } else {
                high = middle;
            }
        }
        roots.push((low + high) * 0.5);
    }

    roots.retain(|root| root.is_finite());
    roots.sort_by(Scalar::total_cmp);
    roots.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-10);
    roots
}

fn evaluate_polynomial(coefficients: &[Scalar], x: Scalar) -> Scalar {
    coefficients
        .iter()
        .rev()
        .fold(0.0, |value, coefficient| value * x + coefficient)
}

fn vector_from_points(point: Point2) -> Vector2 {
    Vector2::new(point.x, point.y)
}

fn validate_segment(segment: Segment) -> CoreResult<()> {
    let valid = match segment {
        Segment::Line(line) => line.start.is_finite() && line.end.is_finite(),
        Segment::Quadratic(curve) => {
            curve.p0.is_finite() && curve.p1.is_finite() && curve.p2.is_finite()
        }
        Segment::Cubic(curve) => {
            curve.p0.is_finite()
                && curve.p1.is_finite()
                && curve.p2.is_finite()
                && curve.p3.is_finite()
        }
        Segment::Arc(arc) => {
            arc.center.is_finite()
                && arc.radius_x.is_finite()
                && arc.radius_y.is_finite()
                && arc.radius_x > 0.0
                && arc.radius_y > 0.0
                && arc.rotation.as_radians().is_finite()
                && arc.start_angle.as_radians().is_finite()
                && arc.sweep_angle.as_radians().is_finite()
        }
    };

    if valid {
        Ok(())
    } else {
        Err(CoreError::InvalidGeometry)
    }
}
