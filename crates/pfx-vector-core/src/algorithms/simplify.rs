use crate::{
    CoreError, CoreResult, CubicBezier, LineSegment, Path, PathBuilder, Point2, Scalar, Segment,
    Tolerance, Vector2, flatten_path,
};

const MAX_FIT_DEPTH: u32 = 32;
const MAX_REPARAMETERIZATION_STEPS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CleanupOptions {
    pub point_tolerance: Scalar,
    pub collinear_tolerance: Scalar,
}

impl Default for CleanupOptions {
    fn default() -> Self {
        Self {
            point_tolerance: 1.0e-9,
            collinear_tolerance: 1.0e-7,
        }
    }
}

impl CleanupOptions {
    pub fn validate(self) -> CoreResult<()> {
        if !self.point_tolerance.is_finite()
            || !self.collinear_tolerance.is_finite()
            || self.point_tolerance < 0.0
            || self.collinear_tolerance < 0.0
        {
            return Err(CoreError::InvalidNumber);
        }

        Ok(())
    }
}

pub fn cleanup_path(
    path: &Path,
    options: CleanupOptions,
    tolerance: Tolerance,
) -> CoreResult<Path> {
    options.validate()?;

    let mut builder = PathBuilder::with_tolerance(tolerance);

    for subpath in path.subpaths() {
        if subpath
            .segments()
            .iter()
            .all(|segment| matches!(segment, Segment::Line(_)))
        {
            let mut points = Vec::with_capacity(subpath.segments().len() + 1);
            points.push(subpath.start());
            points.extend(subpath.segments().iter().map(|segment| segment.end()));
            let cleaned = cleanup_polyline_points(&points, subpath.is_closed(), options, tolerance);

            append_polyline(&mut builder, &cleaned, subpath.is_closed())?;
            continue;
        }

        builder.move_to(subpath.start())?;
        let mut pending_line_start: Option<Point2> = None;
        let mut pending_line_end: Option<Point2> = None;

        for &segment in subpath.segments() {
            let simplified =
                simplify_nearly_linear_segment(segment, options.collinear_tolerance, tolerance);

            match simplified {
                Segment::Line(line) => {
                    if line.length() <= options.point_tolerance
                        && line.start.almost_eq(line.end, tolerance)
                    {
                        continue;
                    }

                    match (pending_line_start, pending_line_end) {
                        (Some(start), Some(end))
                            if can_merge_lines(
                                start,
                                end,
                                line.end,
                                options.collinear_tolerance,
                                tolerance,
                            ) =>
                        {
                            pending_line_end = Some(line.end);
                        }
                        (Some(start), Some(end)) => {
                            builder.line_to(end)?;
                            pending_line_start = Some(end);
                            pending_line_end = Some(line.end);
                            let _ = start;
                        }
                        _ => {
                            pending_line_start = Some(line.start);
                            pending_line_end = Some(line.end);
                        }
                    }
                }
                other => {
                    if let Some(end) = pending_line_end.take() {
                        builder.line_to(end)?;
                        pending_line_start = None;
                    }
                    append_segment(&mut builder, other)?;
                }
            }
        }

        if let Some(end) = pending_line_end {
            builder.line_to(end)?;
        }

        if subpath.is_closed() {
            builder.close()?;
        }
    }

    builder.finish()
}

pub fn simplify_path(path: &Path, max_deviation: Scalar, tolerance: Tolerance) -> CoreResult<Path> {
    validate_positive_error(max_deviation)?;

    let mut flatten_tolerance = tolerance;
    flatten_tolerance.flatness = flatten_tolerance.flatness.min(max_deviation * 0.25);
    let flattened = flatten_path(path, flatten_tolerance)?;
    let mut builder = PathBuilder::with_tolerance(tolerance);

    for subpath in flattened {
        let points = deduplicate_points(&subpath.points, tolerance);
        let simplified =
            simplify_polyline_points(&points, subpath.closed, max_deviation, tolerance);
        append_polyline(&mut builder, &simplified, subpath.closed)?;
    }

    builder.finish()
}

pub fn fit_path_curves(path: &Path, max_error: Scalar, tolerance: Tolerance) -> CoreResult<Path> {
    validate_positive_error(max_error)?;

    let mut flatten_tolerance = tolerance;
    flatten_tolerance.flatness = flatten_tolerance.flatness.min(max_error * 0.25);
    let flattened = flatten_path(path, flatten_tolerance)?;
    let mut builder = PathBuilder::with_tolerance(tolerance);

    for subpath in flattened {
        let points = deduplicate_points(&subpath.points, tolerance);
        if points.is_empty() {
            continue;
        }

        if points.len() == 1 {
            builder.move_to(points[0])?;
            if subpath.closed {
                builder.close()?;
            }
            continue;
        }

        let curves = fit_polyline_cubics(&points, subpath.closed, max_error, tolerance)?;
        if curves.is_empty() {
            builder.move_to(points[0])?;
            if subpath.closed {
                builder.close()?;
            }
            continue;
        }

        builder.move_to(curves[0].p0)?;
        for curve in curves {
            builder.cubic_to(curve.p1, curve.p2, curve.p3)?;
        }
        if subpath.closed {
            builder.close()?;
        }
    }

    builder.finish()
}

pub fn fit_polyline_cubics(
    points: &[Point2],
    closed: bool,
    max_error: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Vec<CubicBezier>> {
    validate_positive_error(max_error)?;

    let mut clean = deduplicate_points(points, tolerance);
    if closed && clean.len() > 1 && clean[0].almost_eq(*clean.last().expect("non-empty"), tolerance)
    {
        clean.pop();
    }

    if clean.len() < 2 {
        return Ok(Vec::new());
    }

    if closed {
        if clean.len() < 3 {
            return Err(CoreError::DegenerateOperation);
        }
        let first = clean[0];
        clean.push(first);
    }

    let left_tangent = endpoint_tangent(&clean, true, tolerance)?;
    let right_tangent = endpoint_tangent(&clean, false, tolerance)?;
    let mut curves = Vec::new();
    let last = clean.len() - 1;
    let mut context = FitContext {
        points: &clean,
        error_squared: max_error * max_error,
        tolerance,
        output: &mut curves,
    };

    fit_cubic_recursive(
        &mut context,
        0,
        last,
        left_tangent,
        right_tangent,
        MAX_FIT_DEPTH,
    )?;

    Ok(curves)
}

fn validate_positive_error(value: Scalar) -> CoreResult<()> {
    if !value.is_finite() {
        return Err(CoreError::InvalidNumber);
    }
    if value <= 0.0 {
        return Err(CoreError::InvalidGeometry);
    }
    Ok(())
}

fn cleanup_polyline_points(
    points: &[Point2],
    closed: bool,
    options: CleanupOptions,
    tolerance: Tolerance,
) -> Vec<Point2> {
    let mut cleaned = Vec::with_capacity(points.len());

    for &point in points {
        if cleaned
            .last()
            .is_none_or(|previous: &Point2| previous.distance_to(point) > options.point_tolerance)
        {
            cleaned.push(point);
        }
    }

    if closed
        && cleaned.len() > 1
        && cleaned[0].distance_to(*cleaned.last().expect("non-empty")) <= options.point_tolerance
    {
        cleaned.pop();
    }

    if cleaned.len() < 3 {
        return cleaned;
    }

    loop {
        let mut removed = false;
        let start = if closed { 0 } else { 1 };
        let end = if closed {
            cleaned.len()
        } else {
            cleaned.len().saturating_sub(1)
        };

        for index in start..end {
            if cleaned.len() <= if closed { 3 } else { 2 } {
                break;
            }

            let previous = if index == 0 {
                cleaned[cleaned.len() - 1]
            } else {
                cleaned[index - 1]
            };
            let current = cleaned[index];
            let next = cleaned[(index + 1) % cleaned.len()];

            if point_line_distance(current, previous, next, tolerance)
                <= options.collinear_tolerance
                && between_along_line(current, previous, next, tolerance)
            {
                cleaned.remove(index);
                removed = true;
                break;
            }
        }

        if !removed {
            break;
        }
    }

    cleaned
}

fn simplify_nearly_linear_segment(
    segment: Segment,
    max_deviation: Scalar,
    tolerance: Tolerance,
) -> Segment {
    match segment {
        Segment::Quadratic(curve)
            if point_line_distance(curve.p1, curve.p0, curve.p2, tolerance) <= max_deviation =>
        {
            Segment::Line(LineSegment::new(curve.p0, curve.p2))
        }
        Segment::Cubic(curve)
            if point_line_distance(curve.p1, curve.p0, curve.p3, tolerance) <= max_deviation
                && point_line_distance(curve.p2, curve.p0, curve.p3, tolerance)
                    <= max_deviation =>
        {
            Segment::Line(LineSegment::new(curve.p0, curve.p3))
        }
        other => other,
    }
}

fn can_merge_lines(
    start: Point2,
    middle: Point2,
    end: Point2,
    max_deviation: Scalar,
    tolerance: Tolerance,
) -> bool {
    if point_line_distance(middle, start, end, tolerance) > max_deviation {
        return false;
    }

    between_along_line(middle, start, end, tolerance)
}

fn between_along_line(point: Point2, start: Point2, end: Point2, tolerance: Tolerance) -> bool {
    let direction = end - start;
    let length_squared = direction.length_squared();
    if tolerance.nearly_zero(length_squared, length_squared) {
        return point.almost_eq(start, tolerance);
    }

    let t = (point - start).dot(direction) / length_squared;
    t >= -tolerance.relative && t <= 1.0 + tolerance.relative
}

fn point_line_distance(point: Point2, start: Point2, end: Point2, tolerance: Tolerance) -> Scalar {
    let direction = end - start;
    let length = direction.length();
    if tolerance.nearly_zero(length, direction.x.abs().max(direction.y.abs())) {
        return point.distance_to(start);
    }

    ((point - start).cross(direction)).abs() / length
}

fn deduplicate_points(points: &[Point2], tolerance: Tolerance) -> Vec<Point2> {
    let mut result = Vec::with_capacity(points.len());
    for &point in points {
        if result
            .last()
            .is_none_or(|previous: &Point2| !previous.almost_eq(point, tolerance))
        {
            result.push(point);
        }
    }
    result
}

fn simplify_polyline_points(
    points: &[Point2],
    closed: bool,
    max_deviation: Scalar,
    tolerance: Tolerance,
) -> Vec<Point2> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    if !closed {
        return rdp(points, max_deviation, tolerance);
    }

    let ring = if points[0].almost_eq(*points.last().expect("non-empty"), tolerance) {
        &points[..points.len() - 1]
    } else {
        points
    };

    if ring.len() <= 3 {
        return ring.to_vec();
    }

    let (a, b) = farthest_pair(ring);
    let first_chain = cyclic_chain(ring, a, b);
    let second_chain = cyclic_chain(ring, b, a);
    let first = rdp(&first_chain, max_deviation, tolerance);
    let second = rdp(&second_chain, max_deviation, tolerance);

    let mut result = first;
    if second.len() > 2 {
        result.extend_from_slice(&second[1..second.len() - 1]);
    }

    result
}

fn rdp(points: &[Point2], max_deviation: Scalar, tolerance: Tolerance) -> Vec<Point2> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut stack = vec![(0usize, points.len() - 1)];

    while let Some((first, last)) = stack.pop() {
        if last <= first + 1 {
            continue;
        }

        let mut maximum = 0.0;
        let mut split = None;

        for index in first + 1..last {
            let distance =
                point_segment_distance(points[index], points[first], points[last], tolerance);
            if distance > maximum {
                maximum = distance;
                split = Some(index);
            }
        }

        if maximum > max_deviation {
            let index = split.expect("positive maximum has a split");
            keep[index] = true;
            stack.push((first, index));
            stack.push((index, last));
        }
    }

    points
        .iter()
        .copied()
        .zip(keep)
        .filter_map(|(point, keep)| keep.then_some(point))
        .collect()
}

fn point_segment_distance(
    point: Point2,
    start: Point2,
    end: Point2,
    tolerance: Tolerance,
) -> Scalar {
    let direction = end - start;
    let length_squared = direction.length_squared();
    if tolerance.nearly_zero(length_squared, length_squared) {
        return point.distance_to(start);
    }

    let t = ((point - start).dot(direction) / length_squared).clamp(0.0, 1.0);
    point.distance_to(start + direction * t)
}

fn farthest_pair(points: &[Point2]) -> (usize, usize) {
    let mut best = (0usize, 1usize);
    let mut best_distance = points[0].distance_squared_to(points[1]);

    for left in 0..points.len() {
        for right in left + 1..points.len() {
            let distance = points[left].distance_squared_to(points[right]);
            if distance > best_distance {
                best = (left, right);
                best_distance = distance;
            }
        }
    }

    best
}

fn cyclic_chain(points: &[Point2], start: usize, end: usize) -> Vec<Point2> {
    let mut result = Vec::new();
    let mut index = start;
    loop {
        result.push(points[index]);
        if index == end {
            break;
        }
        index = (index + 1) % points.len();
    }
    result
}

struct FitContext<'a> {
    points: &'a [Point2],
    error_squared: Scalar,
    tolerance: Tolerance,
    output: &'a mut Vec<CubicBezier>,
}

fn fit_cubic_recursive(
    context: &mut FitContext<'_>,
    first: usize,
    last: usize,
    left_tangent: Vector2,
    right_tangent: Vector2,
    depth: u32,
) -> CoreResult<()> {
    let points = context.points;
    let tolerance = context.tolerance;
    let error_squared = context.error_squared;
    if depth == 0 {
        return Err(CoreError::IterationLimit);
    }

    let count = last - first + 1;
    if count == 2 {
        let distance = points[first].distance_to(points[last]) / 3.0;
        context.output.push(CubicBezier::new(
            points[first],
            points[first] + left_tangent * distance,
            points[last] + right_tangent * distance,
            points[last],
        ));
        return Ok(());
    }

    let mut parameters = chord_length_parameters(points, first, last, tolerance);
    let mut curve = generate_bezier(
        points,
        first,
        last,
        &parameters,
        left_tangent,
        right_tangent,
        tolerance,
    );
    let (mut maximum_error, mut split) = maximum_fit_error(points, first, last, &parameters, curve);

    if maximum_error <= error_squared {
        context.output.push(curve);
        return Ok(());
    }

    if maximum_error <= error_squared * 16.0 {
        for _ in 0..MAX_REPARAMETERIZATION_STEPS {
            let candidate_parameters =
                reparameterize(points, first, last, &parameters, curve, tolerance);
            if !parameters_are_strictly_increasing(&candidate_parameters) {
                break;
            }
            parameters = candidate_parameters;
            curve = generate_bezier(
                points,
                first,
                last,
                &parameters,
                left_tangent,
                right_tangent,
                tolerance,
            );
            let result = maximum_fit_error(points, first, last, &parameters, curve);
            maximum_error = result.0;
            split = result.1;

            if maximum_error <= error_squared {
                context.output.push(curve);
                return Ok(());
            }
        }
    }

    if split <= first || split >= last {
        split = first + count / 2;
    }

    let center_tangent = center_tangent(points, split, tolerance)?;

    fit_cubic_recursive(
        context,
        first,
        split,
        left_tangent,
        center_tangent,
        depth - 1,
    )?;
    fit_cubic_recursive(
        context,
        split,
        last,
        -center_tangent,
        right_tangent,
        depth - 1,
    )
}

fn chord_length_parameters(
    points: &[Point2],
    first: usize,
    last: usize,
    tolerance: Tolerance,
) -> Vec<Scalar> {
    let mut parameters = Vec::with_capacity(last - first + 1);
    parameters.push(0.0);

    let mut total = 0.0;
    for index in first + 1..=last {
        total += points[index - 1].distance_to(points[index]);
        parameters.push(total);
    }

    if tolerance.nearly_zero(total, total) {
        let denominator = (last - first) as Scalar;
        for (index, value) in parameters.iter_mut().enumerate() {
            *value = index as Scalar / denominator;
        }
        return parameters;
    }

    for value in &mut parameters {
        *value /= total;
    }

    parameters
}

fn generate_bezier(
    points: &[Point2],
    first: usize,
    last: usize,
    parameters: &[Scalar],
    left_tangent: Vector2,
    right_tangent: Vector2,
    tolerance: Tolerance,
) -> CubicBezier {
    let p0 = points[first];
    let p3 = points[last];

    let mut c00 = 0.0;
    let mut c01 = 0.0;
    let mut c11 = 0.0;
    let mut x0 = 0.0;
    let mut x1 = 0.0;

    for (local_index, &u) in parameters.iter().enumerate() {
        let point = points[first + local_index];
        let (b0, b1, b2, b3) = bernstein(u);
        let a0 = left_tangent * b1;
        let a1 = right_tangent * b2;
        let base = Point2::new(
            p0.x * (b0 + b1) + p3.x * (b2 + b3),
            p0.y * (b0 + b1) + p3.y * (b2 + b3),
        );
        let residual = point - base;

        c00 += a0.dot(a0);
        c01 += a0.dot(a1);
        c11 += a1.dot(a1);
        x0 += a0.dot(residual);
        x1 += a1.dot(residual);
    }

    let determinant = c00 * c11 - c01 * c01;
    let chord = p0.distance_to(p3);
    let fallback = chord / 3.0;
    let minimum_alpha = (tolerance.absolute + tolerance.relative * chord.max(1.0)).max(1.0e-12);

    let (alpha_left, alpha_right) = if determinant.abs() > minimum_alpha * minimum_alpha {
        (
            (x0 * c11 - x1 * c01) / determinant,
            (c00 * x1 - c01 * x0) / determinant,
        )
    } else {
        (fallback, fallback)
    };

    let alpha_left = if alpha_left.is_finite() && alpha_left > minimum_alpha {
        alpha_left
    } else {
        fallback
    };
    let alpha_right = if alpha_right.is_finite() && alpha_right > minimum_alpha {
        alpha_right
    } else {
        fallback
    };

    CubicBezier::new(
        p0,
        p0 + left_tangent * alpha_left,
        p3 + right_tangent * alpha_right,
        p3,
    )
}

fn maximum_fit_error(
    points: &[Point2],
    first: usize,
    last: usize,
    parameters: &[Scalar],
    curve: CubicBezier,
) -> (Scalar, usize) {
    let mut maximum = 0.0;
    let mut split = first + (last - first) / 2;

    for local_index in 1..parameters.len() - 1 {
        let point = points[first + local_index];
        let distance = curve
            .point_at(parameters[local_index])
            .distance_squared_to(point);
        if distance > maximum {
            maximum = distance;
            split = first + local_index;
        }
    }

    (maximum, split)
}

fn reparameterize(
    points: &[Point2],
    first: usize,
    last: usize,
    parameters: &[Scalar],
    curve: CubicBezier,
    tolerance: Tolerance,
) -> Vec<Scalar> {
    parameters
        .iter()
        .enumerate()
        .map(|(local_index, &u)| {
            if local_index == 0 {
                0.0
            } else if first + local_index == last {
                1.0
            } else {
                newton_parameter(curve, points[first + local_index], u, tolerance)
            }
        })
        .collect()
}

fn newton_parameter(
    curve: CubicBezier,
    point: Point2,
    parameter: Scalar,
    tolerance: Tolerance,
) -> Scalar {
    let q = curve.point_at(parameter);
    let q1 = curve.derivative_at(parameter);
    let q2 = cubic_second_derivative(curve, parameter);
    let difference = q - point;
    let numerator = difference.dot(q1);
    let denominator = q1.dot(q1) + difference.dot(q2);

    if tolerance.nearly_zero(denominator, q1.length_squared().max(q2.length_squared())) {
        return parameter;
    }

    (parameter - numerator / denominator).clamp(0.0, 1.0)
}

fn cubic_second_derivative(curve: CubicBezier, t: Scalar) -> Vector2 {
    let first = (curve.p2 - curve.p1) - (curve.p1 - curve.p0);
    let second = (curve.p3 - curve.p2) - (curve.p2 - curve.p1);
    first * (6.0 * (1.0 - t)) + second * (6.0 * t)
}

fn parameters_are_strictly_increasing(parameters: &[Scalar]) -> bool {
    parameters.windows(2).all(|window| window[1] > window[0])
}

fn endpoint_tangent(
    points: &[Point2],
    at_start: bool,
    tolerance: Tolerance,
) -> CoreResult<Vector2> {
    if at_start {
        for index in 1..points.len() {
            if let Ok(tangent) = (points[index] - points[0]).normalized(tolerance) {
                return Ok(tangent);
            }
        }
    } else {
        let last = points.len() - 1;
        for index in (0..last).rev() {
            if let Ok(tangent) = (points[index] - points[last]).normalized(tolerance) {
                return Ok(tangent);
            }
        }
    }

    Err(CoreError::DegenerateOperation)
}

fn center_tangent(points: &[Point2], center: usize, tolerance: Tolerance) -> CoreResult<Vector2> {
    if center > 0 && center + 1 < points.len() {
        if let Ok(tangent) = (points[center - 1] - points[center + 1]).normalized(tolerance) {
            return Ok(tangent);
        }
    }

    if center > 0 {
        if let Ok(tangent) = (points[center - 1] - points[center]).normalized(tolerance) {
            return Ok(tangent);
        }
    }

    if center + 1 < points.len() {
        if let Ok(tangent) = (points[center] - points[center + 1]).normalized(tolerance) {
            return Ok(tangent);
        }
    }

    Err(CoreError::DegenerateOperation)
}

fn bernstein(t: Scalar) -> (Scalar, Scalar, Scalar, Scalar) {
    let mt = 1.0 - t;
    (mt * mt * mt, 3.0 * t * mt * mt, 3.0 * t * t * mt, t * t * t)
}

fn append_polyline(builder: &mut PathBuilder, points: &[Point2], closed: bool) -> CoreResult<()> {
    let Some(&first) = points.first() else {
        return Ok(());
    };

    builder.move_to(first)?;
    for &point in &points[1..] {
        builder.line_to(point)?;
    }
    if closed {
        builder.close()?;
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
