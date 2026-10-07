use crate::{
    Angle, BooleanOperation, Circle, CoreError, CoreResult, CubicBezier, EllipticalArc, FillRule,
    LineSegment, Path, PathBuilder, Point2, Scalar, Segment, StrokeCap, StrokeJoin, StrokeStyle,
    Tolerance, Vector2, boolean_paths_with_fill_rules, boolean_union, circle_to_path, dash_path,
    normalize_self_intersections, normalized_dash_pattern,
};

const MAX_OFFSET_DEPTH: u32 = 18;
const SAMPLE_PARAMETERS: [Scalar; 3] = [0.25, 0.5, 0.75];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OffsetStyle {
    pub join: StrokeJoin,
    pub miter_limit: Scalar,
}

impl Default for OffsetStyle {
    fn default() -> Self {
        Self {
            join: StrokeJoin::Miter,
            miter_limit: 4.0,
        }
    }
}

impl OffsetStyle {
    pub fn validate(self) -> CoreResult<()> {
        if !self.miter_limit.is_finite() || self.miter_limit < 0.0 {
            return Err(CoreError::InvalidGeometry);
        }

        Ok(())
    }
}

pub fn offset_path(
    path: &Path,
    distance: Scalar,
    style: OffsetStyle,
    tolerance: Tolerance,
) -> CoreResult<Path> {
    offset_path_with_fill_rule(path, FillRule::NonZero, distance, style, tolerance)
}

pub fn offset_path_with_fill_rule(
    path: &Path,
    fill_rule: FillRule,
    distance: Scalar,
    style: OffsetStyle,
    tolerance: Tolerance,
) -> CoreResult<Path> {
    if !distance.is_finite() {
        return Err(CoreError::InvalidNumber);
    }
    style.validate()?;

    if path.subpaths().iter().any(|subpath| !subpath.is_closed()) {
        return Err(CoreError::UnsupportedCase);
    }

    if distance == 0.0 || path.is_empty() {
        return Ok(path.clone());
    }

    let normalized = normalize_self_intersections(path, fill_rule, tolerance)?;
    if normalized.is_empty() {
        return Ok(Path::new());
    }

    let outline_style = StrokeStyle {
        width: distance.abs() * 2.0,
        cap: StrokeCap::Butt,
        join: style.join,
        miter_limit: style.miter_limit,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };
    let outline = outline_path(&normalized, &outline_style, tolerance)?;

    boolean_paths_with_fill_rules(
        &normalized,
        FillRule::NonZero,
        &outline,
        FillRule::NonZero,
        if distance > 0.0 {
            BooleanOperation::Union
        } else {
            BooleanOperation::Difference
        },
        tolerance,
    )
}

pub fn stroke_to_path(path: &Path, style: &StrokeStyle, tolerance: Tolerance) -> CoreResult<Path> {
    outline_path(path, style, tolerance)
}

pub fn outline_path(path: &Path, style: &StrokeStyle, tolerance: Tolerance) -> CoreResult<Path> {
    style.validate()?;

    if style.width == 0.0 || path.is_empty() {
        return Ok(Path::new());
    }

    if !normalized_dash_pattern(style).is_empty() {
        let dashed = dash_path(path, style, tolerance)?;
        let mut solid_style = style.clone();
        solid_style.dash_array.clear();
        solid_style.dash_offset = 0.0;
        return outline_path(&dashed, &solid_style, tolerance);
    }

    let half_width = style.width * 0.5;
    let mut result = Path::new();

    for subpath in path.subpaths() {
        let mut segments = subpath.segments().to_vec();

        if subpath.is_closed() && !subpath.end().almost_eq(subpath.start(), tolerance) {
            segments.push(Segment::Line(LineSegment::new(
                subpath.end(),
                subpath.start(),
            )));
        }

        if segments.is_empty() {
            continue;
        }

        for &segment in &segments {
            let ribbon = segment_ribbon(segment, half_width, tolerance)?;
            result = union_component(result, ribbon, tolerance)?;
        }

        let join_count = if subpath.is_closed() {
            segments.len()
        } else {
            segments.len().saturating_sub(1)
        };

        for index in 0..join_count {
            let next_index = (index + 1) % segments.len();
            let vertex = segments[index].end();

            if let Some(join) = join_component(
                segments[index],
                segments[next_index],
                vertex,
                half_width,
                style.join,
                style.miter_limit,
                tolerance,
            )? {
                result = union_component(result, join, tolerance)?;
            }
        }

        if !subpath.is_closed() {
            if let Some(cap) = cap_component(segments[0], true, half_width, style.cap, tolerance)? {
                result = union_component(result, cap, tolerance)?;
            }

            if let Some(cap) = cap_component(
                *segments.last().expect("segments is not empty"),
                false,
                half_width,
                style.cap,
                tolerance,
            )? {
                result = union_component(result, cap, tolerance)?;
            }
        }
    }

    Ok(result)
}

fn union_component(mut result: Path, component: Path, tolerance: Tolerance) -> CoreResult<Path> {
    if component.is_empty() {
        return Ok(result);
    }

    if result.is_empty() {
        return Ok(component);
    }

    result = boolean_union(&result, &component, tolerance)?;
    Ok(result)
}

fn segment_ribbon(segment: Segment, half_width: Scalar, tolerance: Tolerance) -> CoreResult<Path> {
    if let Segment::Arc(arc) = segment
        && is_full_circle(arc, tolerance)
        && tolerance.almost_eq(arc.radius_x, arc.radius_y)
    {
        return circular_arc_ribbon(arc, half_width, tolerance);
    }

    let left = offset_segments(segment, half_width, tolerance)?;
    let right = offset_segments(segment, -half_width, tolerance)?;

    if left.is_empty() || right.is_empty() {
        return Err(CoreError::DegenerateOperation);
    }

    let mut builder = PathBuilder::with_tolerance(tolerance);
    builder.move_to(left[0].start())?;
    append_chain(&mut builder, &left)?;

    let right_end = right
        .last()
        .map(|segment| segment.end())
        .ok_or(CoreError::DegenerateOperation)?;
    builder.line_to(right_end)?;

    for segment in right.iter().rev() {
        append_segment(&mut builder, segment.reversed())?;
    }

    builder.close()?;
    builder.finish()
}

fn circular_arc_ribbon(
    arc: EllipticalArc,
    half_width: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Path> {
    let radius = arc.radius_x;
    let inner = radius - half_width;
    let outer = radius + half_width;

    if inner <= tolerance.absolute {
        return circle_to_path(Circle::new(arc.center, outer)?);
    }

    let outer_arc = EllipticalArc::new(
        arc.center,
        outer,
        outer,
        arc.rotation,
        arc.start_angle,
        arc.sweep_angle,
    );
    let inner_arc = EllipticalArc::new(
        arc.center,
        inner,
        inner,
        arc.rotation,
        arc.start_angle,
        arc.sweep_angle,
    )
    .reversed();

    let mut builder = PathBuilder::with_tolerance(tolerance);
    builder
        .move_to(outer_arc.point_at(0.0))?
        .arc_to(outer_arc)?
        .close()?;
    builder
        .move_to(inner_arc.point_at(0.0))?
        .arc_to(inner_arc)?
        .close()?;
    builder.finish()
}

fn offset_segments(
    segment: Segment,
    distance: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Vec<Segment>> {
    match segment {
        Segment::Line(line) => {
            let tangent = line.direction().normalized(tolerance)?;
            let shift = tangent.perpendicular() * distance;
            Ok(vec![Segment::Line(LineSegment::new(
                line.start + shift,
                line.end + shift,
            ))])
        }
        Segment::Arc(arc)
            if tolerance.almost_eq(arc.radius_x, arc.radius_y)
                && arc.sweep_angle.as_radians().abs() > tolerance.angular =>
        {
            let sweep_sign = arc.sweep_angle.as_radians().signum();
            let radius = arc.radius_x - sweep_sign * distance;

            if radius <= tolerance.absolute {
                return approximate_offset(segment, distance, tolerance);
            }

            Ok(vec![Segment::Arc(EllipticalArc::new(
                arc.center,
                radius,
                radius,
                arc.rotation,
                arc.start_angle,
                arc.sweep_angle,
            ))])
        }
        _ => approximate_offset(segment, distance, tolerance),
    }
}

fn approximate_offset(
    segment: Segment,
    distance: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Vec<Segment>> {
    let mut output = Vec::new();
    fit_offset_interval(
        segment,
        distance,
        0.0,
        1.0,
        tolerance,
        MAX_OFFSET_DEPTH,
        &mut output,
    )?;
    Ok(output)
}

fn fit_offset_interval(
    segment: Segment,
    distance: Scalar,
    t0: Scalar,
    t1: Scalar,
    tolerance: Tolerance,
    depth: u32,
    output: &mut Vec<Segment>,
) -> CoreResult<()> {
    if depth == 0 {
        return Err(CoreError::ToleranceNotMet);
    }

    let p0 = offset_point(segment, t0, distance, tolerance)?;
    let p3 = offset_point(segment, t1, distance, tolerance)?;
    let derivative0 = offset_derivative(segment, t0, distance, tolerance)?;
    let derivative1 = offset_derivative(segment, t1, distance, tolerance)?;
    let span = t1 - t0;

    let candidate = CubicBezier::new(
        p0,
        p0 + derivative0 * (span / 3.0),
        p3 - derivative1 * (span / 3.0),
        p3,
    );

    let allowed_error = offset_error_tolerance(segment, distance, tolerance);
    let mut maximum_error: Scalar = 0.0;

    for u in SAMPLE_PARAMETERS {
        let t = t0 + span * u;
        let expected = offset_point(segment, t, distance, tolerance)?;
        maximum_error = maximum_error.max(candidate.point_at(u).distance_to(expected));
    }

    if maximum_error <= allowed_error {
        output.push(Segment::Cubic(candidate));
        return Ok(());
    }

    let midpoint = (t0 + t1) * 0.5;
    fit_offset_interval(
        segment,
        distance,
        t0,
        midpoint,
        tolerance,
        depth - 1,
        output,
    )?;
    fit_offset_interval(
        segment,
        distance,
        midpoint,
        t1,
        tolerance,
        depth - 1,
        output,
    )
}

fn offset_point(
    segment: Segment,
    t: Scalar,
    distance: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Point2> {
    let tangent = stable_segment_tangent(segment, t, tolerance)?;
    Ok(segment.point_at(t) + tangent.perpendicular() * distance)
}

fn offset_derivative(
    segment: Segment,
    t: Scalar,
    distance: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Vector2> {
    let h = 1.0e-5;
    let t0 = (t - h).max(0.0);
    let t1 = (t + h).min(1.0);

    if t1 <= t0 {
        return Err(CoreError::DegenerateOperation);
    }

    let p0 = offset_point(segment, t0, distance, tolerance)?;
    let p1 = offset_point(segment, t1, distance, tolerance)?;
    Ok((p1 - p0) / (t1 - t0))
}

fn stable_segment_tangent(
    segment: Segment,
    t: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Vector2> {
    let derivative = segment_derivative(segment, t);
    if let Ok(tangent) = derivative.normalized(tolerance) {
        return Ok(tangent);
    }

    let h = 1.0e-6;
    let t0 = (t - h).max(0.0);
    let t1 = (t + h).min(1.0);
    (segment.point_at(t1) - segment.point_at(t0)).normalized(tolerance)
}

fn segment_derivative(segment: Segment, t: Scalar) -> Vector2 {
    match segment {
        Segment::Line(line) => line.direction(),
        Segment::Quadratic(curve) => curve.derivative_at(t),
        Segment::Cubic(curve) => curve.derivative_at(t),
        Segment::Arc(arc) => arc.derivative_at(t),
    }
}

fn offset_error_tolerance(segment: Segment, distance: Scalar, tolerance: Tolerance) -> Scalar {
    let bounds = segment.bounds();
    let scale = bounds
        .width()
        .max(bounds.height())
        .max(distance.abs())
        .max(1.0);
    tolerance
        .flatness
        .max((tolerance.absolute + tolerance.relative * scale) * 16.0)
}

fn join_component(
    previous: Segment,
    next: Segment,
    vertex: Point2,
    half_width: Scalar,
    join: StrokeJoin,
    miter_limit: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Option<Path>> {
    let previous_tangent = stable_segment_tangent(previous, 1.0, tolerance)?;
    let next_tangent = stable_segment_tangent(next, 0.0, tolerance)?;
    let cross = previous_tangent.cross(next_tangent);
    let dot = previous_tangent.dot(next_tangent).clamp(-1.0, 1.0);

    if cross.abs() <= tolerance.angular.max(1.0e-12) {
        if dot > 0.0 {
            return Ok(None);
        }

        return Ok(Some(circle_to_path(Circle::new(vertex, half_width)?)?));
    }

    let turn_angle = cross.atan2(dot);
    let side_distance = if cross > 0.0 { -half_width } else { half_width };
    let previous_outer = vertex + previous_tangent.perpendicular() * side_distance;
    let next_outer = vertex + next_tangent.perpendicular() * side_distance;

    let path = match join {
        StrokeJoin::Bevel => polygon_path(&[vertex, previous_outer, next_outer], tolerance)?,
        StrokeJoin::Round => {
            round_sector(vertex, previous_outer, half_width, turn_angle, tolerance)?
        }
        StrokeJoin::Miter => {
            if let Some(miter) = line_intersection(
                previous_outer,
                previous_tangent,
                next_outer,
                next_tangent,
                tolerance,
            ) {
                let ratio = vertex.distance_to(miter) / half_width.max(tolerance.absolute);
                if ratio <= miter_limit {
                    polygon_path(&[vertex, previous_outer, miter, next_outer], tolerance)?
                } else {
                    polygon_path(&[vertex, previous_outer, next_outer], tolerance)?
                }
            } else {
                polygon_path(&[vertex, previous_outer, next_outer], tolerance)?
            }
        }
    };

    Ok(Some(path))
}

fn cap_component(
    segment: Segment,
    at_start: bool,
    half_width: Scalar,
    cap: StrokeCap,
    tolerance: Tolerance,
) -> CoreResult<Option<Path>> {
    if cap == StrokeCap::Butt {
        return Ok(None);
    }

    let parameter = if at_start { 0.0 } else { 1.0 };
    let center = segment.point_at(parameter);
    let tangent = stable_segment_tangent(segment, parameter, tolerance)?;
    let normal = tangent.perpendicular();
    let left = center + normal * half_width;
    let right = center - normal * half_width;

    let path = match cap {
        StrokeCap::Butt => return Ok(None),
        StrokeCap::Square => {
            let extension = if at_start {
                -tangent * half_width
            } else {
                tangent * half_width
            };
            polygon_path(
                &[left, right, right + extension, left + extension],
                tolerance,
            )?
        }
        StrokeCap::Round => {
            if at_start {
                round_sector(center, right, half_width, -core::f64::consts::PI, tolerance)?
            } else {
                round_sector(center, left, half_width, -core::f64::consts::PI, tolerance)?
            }
        }
    };

    Ok(Some(path))
}

fn round_sector(
    center: Point2,
    start: Point2,
    radius: Scalar,
    sweep: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Path> {
    let vector = start - center;
    let start_angle = vector.y.atan2(vector.x);
    let arc = EllipticalArc::new(
        center,
        radius,
        radius,
        Angle::radians(0.0),
        Angle::radians(start_angle),
        Angle::radians(sweep),
    );

    let mut builder = PathBuilder::with_tolerance(tolerance);
    builder
        .move_to(center)?
        .line_to(arc.point_at(0.0))?
        .arc_to(arc)?
        .close()?;
    builder.finish()
}

fn polygon_path(points: &[Point2], tolerance: Tolerance) -> CoreResult<Path> {
    if points.len() < 3 {
        return Ok(Path::new());
    }

    let mut builder = PathBuilder::with_tolerance(tolerance);
    builder.move_to(points[0])?;
    for &point in &points[1..] {
        builder.line_to(point)?;
    }
    builder.close()?;
    builder.finish()
}

fn line_intersection(
    point_a: Point2,
    direction_a: Vector2,
    point_b: Point2,
    direction_b: Vector2,
    tolerance: Tolerance,
) -> Option<Point2> {
    let denominator = direction_a.cross(direction_b);
    let scale = direction_a.length() * direction_b.length();

    if tolerance.nearly_zero(denominator, scale) {
        return None;
    }

    let parameter = (point_b - point_a).cross(direction_b) / denominator;
    Some(point_a + direction_a * parameter)
}

fn append_chain(builder: &mut PathBuilder, segments: &[Segment]) -> CoreResult<()> {
    for &segment in segments {
        append_segment(builder, segment)?;
    }

    Ok(())
}

fn append_segment(builder: &mut PathBuilder, segment: Segment) -> CoreResult<()> {
    match segment {
        Segment::Line(line) => {
            builder.line_to(line.end)?;
        }
        Segment::Quadratic(curve) => {
            builder.quad_to(curve.p1, curve.p2)?;
        }
        Segment::Cubic(curve) => {
            builder.cubic_to(curve.p1, curve.p2, curve.p3)?;
        }
        Segment::Arc(arc) => {
            builder.arc_to(arc)?;
        }
    }

    Ok(())
}

fn is_full_circle(arc: EllipticalArc, tolerance: Tolerance) -> bool {
    arc.sweep_angle.as_radians().abs() >= core::f64::consts::TAU - tolerance.angular.max(1.0e-12)
}
