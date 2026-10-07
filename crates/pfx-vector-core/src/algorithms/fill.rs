use crate::numeric::{dedup_sorted, solve_cubic, solve_quadratic};
use crate::{
    CoreError, CoreResult, EllipticalArc, LineSegment, Path, Point2, Scalar, Segment, Tolerance,
    closest_point_on_segment,
};

const SIDE_SAMPLE_START: Scalar = 1.0e-8;
const SIDE_SAMPLE_MAX: Scalar = 1.0e-2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointClassification {
    Outside,
    Inside,
    Boundary,
}

pub fn classify_point(
    path: &Path,
    point: Point2,
    rule: FillRule,
    tolerance: Tolerance,
) -> CoreResult<PointClassification> {
    if !point.is_finite() {
        return Err(CoreError::InvalidNumber);
    }

    let mut winding = 0i32;
    let boundary_tolerance = point_tolerance(point, tolerance);

    for subpath in path.subpaths() {
        let mut segments = subpath.segments().to_vec();
        if subpath.is_closed() && !subpath.end().almost_eq(subpath.start(), tolerance) {
            segments.push(Segment::Line(LineSegment::new(
                subpath.end(),
                subpath.start(),
            )));
        }

        for &segment in &segments {
            if segment.bounds().distance_squared_to_point(point).sqrt() <= boundary_tolerance {
                let nearest = closest_point_on_segment(segment, point, tolerance)?;
                if nearest.distance <= boundary_tolerance {
                    return Ok(PointClassification::Boundary);
                }
            }
        }

        let ray_y = point.y + winding_ray_offset(point, tolerance);
        let mut endpoint_events: Vec<(Scalar, i8)> = Vec::new();

        for (segment_index, &segment) in segments.iter().enumerate() {
            let mut roots = horizontal_ray_parameters(segment, ray_y, tolerance);
            dedup_sorted(&mut roots, tolerance);

            for root in roots {
                if root < -tolerance.absolute || root > 1.0 + tolerance.absolute {
                    continue;
                }
                let parameter = root.clamp(0.0, 1.0);
                let intersection = segment.point_at(parameter);
                if intersection.x <= point.x {
                    continue;
                }

                let before = crossing_side_before(
                    &segments,
                    segment_index,
                    parameter,
                    ray_y,
                    subpath.is_closed(),
                    tolerance,
                );
                let after = crossing_side_after(
                    &segments,
                    segment_index,
                    parameter,
                    ray_y,
                    subpath.is_closed(),
                    tolerance,
                );

                let direction = match (before, after) {
                    (Some(-1), Some(1)) => 1,
                    (Some(1), Some(-1)) => -1,
                    _ => 0,
                };
                if direction == 0 {
                    continue;
                }

                let endpoint = parameter == 0.0 || parameter == 1.0;
                if endpoint
                    && endpoint_events.iter().any(|(x, existing_direction)| {
                        *existing_direction == direction && tolerance.almost_eq(*x, intersection.x)
                    })
                {
                    continue;
                }
                if endpoint {
                    endpoint_events.push((intersection.x, direction));
                }

                if (point.x + 2.0).abs() < 0.01 && point.y.abs() < 0.01 {
                    eprintln!(
                        "FILL_EVENT q=({:.12},{:.12}) ray={:.12} seg={} t={:.15} x={:.12} dir={} endpoint={}",
                        point.x,
                        point.y,
                        ray_y,
                        segment_index,
                        parameter,
                        intersection.x,
                        direction,
                        endpoint,
                    );
                }

                winding += i32::from(direction);
            }
        }
    }

    if (point.x + 2.0).abs() < 0.01 && point.y.abs() < 0.01 {
        eprintln!("FILL_RESULT q=({:.12},{:.12}) winding={winding}", point.x, point.y);
    }

    Ok(match rule {
        FillRule::EvenOdd if winding.unsigned_abs() % 2 == 1 => PointClassification::Inside,
        FillRule::NonZero if winding != 0 => PointClassification::Inside,
        _ => PointClassification::Outside,
    })
}

pub fn contains_point(
    path: &Path,
    point: Point2,
    rule: FillRule,
    tolerance: Tolerance,
) -> CoreResult<bool> {
    Ok(classify_point(path, point, rule, tolerance)? != PointClassification::Outside)
}

fn horizontal_ray_parameters(segment: Segment, ray_y: Scalar, tolerance: Tolerance) -> Vec<Scalar> {
    let mut roots = match segment {
        Segment::Line(line) => {
            let delta_y = line.end.y - line.start.y;
            if tolerance.nearly_zero(delta_y, line.start.y.abs().max(line.end.y.abs()).max(1.0)) {
                Vec::new()
            } else {
                vec![(ray_y - line.start.y) / delta_y]
            }
        }
        Segment::Quadratic(curve) => {
            let a = curve.p0.y - 2.0 * curve.p1.y + curve.p2.y;
            let b = 2.0 * (curve.p1.y - curve.p0.y);
            let c = curve.p0.y - ray_y;
            solve_quadratic(a, b, c)
        }
        Segment::Cubic(curve) => {
            let a = -curve.p0.y + 3.0 * curve.p1.y - 3.0 * curve.p2.y + curve.p3.y;
            let b = 3.0 * curve.p0.y - 6.0 * curve.p1.y + 3.0 * curve.p2.y;
            let c = -3.0 * curve.p0.y + 3.0 * curve.p1.y;
            let d = curve.p0.y - ray_y;
            solve_cubic(a, b, c, d)
        }
        Segment::Arc(arc) => arc_horizontal_parameters(arc, ray_y, tolerance),
    };

    roots.retain(|root| root.is_finite());
    roots.sort_by(Scalar::total_cmp);
    roots
}

fn arc_horizontal_parameters(
    arc: EllipticalArc,
    ray_y: Scalar,
    tolerance: Tolerance,
) -> Vec<Scalar> {
    let rotation = arc.rotation.as_radians();
    let (sin_rotation, cos_rotation) = rotation.sin_cos();
    let cosine_coefficient = arc.radius_x * sin_rotation;
    let sine_coefficient = arc.radius_y * cos_rotation;
    let radius = cosine_coefficient.hypot(sine_coefficient);

    if tolerance.nearly_zero(radius, arc.radius_x.abs().max(arc.radius_y.abs()).max(1.0)) {
        return Vec::new();
    }

    let value = (ray_y - arc.center.y) / radius;
    let numerical = tolerance.absolute
        + tolerance.relative * ray_y.abs().max(arc.center.y.abs()).max(radius).max(1.0);
    if value > 1.0 + numerical || value < -1.0 - numerical {
        return Vec::new();
    }

    let phase = sine_coefficient.atan2(cosine_coefficient);
    let offset = value.clamp(-1.0, 1.0).acos();
    let start = arc.start_angle.as_radians();
    let sweep = arc.sweep_angle.as_radians();

    if sweep == 0.0 {
        return Vec::new();
    }

    let lower = start.min(start + sweep);
    let upper = start.max(start + sweep);
    let mut roots = Vec::new();

    append_arc_solution(&mut roots, phase + offset, start, sweep, lower, upper);
    append_arc_solution(&mut roots, phase - offset, start, sweep, lower, upper);

    roots
}

fn append_arc_solution(
    roots: &mut Vec<Scalar>,
    base_angle: Scalar,
    start: Scalar,
    sweep: Scalar,
    lower: Scalar,
    upper: Scalar,
) {
    let first_turn = ((lower - base_angle) / core::f64::consts::TAU).ceil() as i64;
    let last_turn = ((upper - base_angle) / core::f64::consts::TAU).floor() as i64;

    for turn in first_turn..=last_turn {
        let angle = base_angle + turn as Scalar * core::f64::consts::TAU;
        roots.push((angle - start) / sweep);
    }
}

fn crossing_side_before(
    segments: &[Segment],
    segment_index: usize,
    parameter: Scalar,
    ray_y: Scalar,
    closed: bool,
    tolerance: Tolerance,
) -> Option<i8> {
    if parameter > 0.0 {
        return sample_side(segments[segment_index], parameter, -1.0, ray_y, tolerance);
    }

    if segment_index > 0 {
        return sample_side(segments[segment_index - 1], 1.0, -1.0, ray_y, tolerance);
    }

    if closed {
        return segments
            .last()
            .and_then(|segment| sample_side(*segment, 1.0, -1.0, ray_y, tolerance));
    }

    None
}

fn crossing_side_after(
    segments: &[Segment],
    segment_index: usize,
    parameter: Scalar,
    ray_y: Scalar,
    closed: bool,
    tolerance: Tolerance,
) -> Option<i8> {
    if parameter < 1.0 {
        return sample_side(segments[segment_index], parameter, 1.0, ray_y, tolerance);
    }

    if segment_index + 1 < segments.len() {
        return sample_side(segments[segment_index + 1], 0.0, 1.0, ray_y, tolerance);
    }

    if closed {
        return segments
            .first()
            .and_then(|segment| sample_side(*segment, 0.0, 1.0, ray_y, tolerance));
    }

    None
}

fn sample_side(
    segment: Segment,
    parameter: Scalar,
    direction: Scalar,
    ray_y: Scalar,
    tolerance: Tolerance,
) -> Option<i8> {
    let numerical = tolerance.absolute
        + tolerance.relative
            * ray_y
                .abs()
                .max(segment.start().y.abs())
                .max(segment.end().y.abs())
                .max(1.0);
    let mut delta = SIDE_SAMPLE_START;

    while delta <= SIDE_SAMPLE_MAX {
        let sample_parameter = (parameter + direction * delta).clamp(0.0, 1.0);
        if sample_parameter != parameter {
            let value = segment.point_at(sample_parameter).y - ray_y;
            if value > numerical {
                return Some(1);
            }
            if value < -numerical {
                return Some(-1);
            }
        }
        delta *= 10.0;
    }

    let derivative_y = segment_derivative_y(segment, parameter);
    if derivative_y.abs() > numerical {
        let signed = derivative_y * direction;
        return Some(if signed > 0.0 { 1 } else { -1 });
    }

    None
}

fn segment_derivative_y(segment: Segment, parameter: Scalar) -> Scalar {
    match segment {
        Segment::Line(line) => line.direction().y,
        Segment::Quadratic(curve) => curve.derivative_at(parameter).y,
        Segment::Cubic(curve) => curve.derivative_at(parameter).y,
        Segment::Arc(arc) => arc.derivative_at(parameter).y,
    }
}

fn winding_ray_offset(point: Point2, tolerance: Tolerance) -> Scalar {
    point_tolerance(point, tolerance) * 0.25
}

fn point_tolerance(point: Point2, tolerance: Tolerance) -> Scalar {
    tolerance.absolute + tolerance.relative * point.x.abs().max(point.y.abs()).max(1.0)
}
