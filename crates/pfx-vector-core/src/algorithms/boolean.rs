use crate::{
    Angle, CoreError, CoreResult, CubicBezier, EllipticalArc, FillRule, Intersection, LineSegment,
    Path, PathBuilder, PathSpatialIndex, Point2, PointClassification, Scalar, Segment, Tolerance,
    Vector2, intersect_segments, spatial_cross_candidate_pairs, spatial_self_candidate_pairs,
};

const PARAMETER_EPSILON: Scalar = 1.0e-10;
const MAX_PROBE_ATTEMPTS: usize = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BooleanOperation {
    Union,
    Intersection,
    Difference,
    Xor,
}

#[derive(Clone, Copy, Debug)]
struct WorkingSegment {
    segment: Segment,
}

#[derive(Clone, Copy, Debug)]
struct BoundaryFragment {
    segment: Segment,
}

#[derive(Clone, Copy)]
struct BooleanContext<'a> {
    a: &'a PathSpatialIndex,
    fill_a: FillRule,
    b: &'a PathSpatialIndex,
    fill_b: FillRule,
    operation: BooleanOperation,
    tolerance: Tolerance,
}

#[must_use]
fn operation_value(operation: BooleanOperation, inside_a: bool, inside_b: bool) -> bool {
    match operation {
        BooleanOperation::Union => inside_a || inside_b,
        BooleanOperation::Intersection => inside_a && inside_b,
        BooleanOperation::Difference => inside_a && !inside_b,
        BooleanOperation::Xor => inside_a != inside_b,
    }
}

pub fn boolean_union(a: &Path, b: &Path, tolerance: Tolerance) -> CoreResult<Path> {
    boolean_paths(a, b, BooleanOperation::Union, tolerance)
}

pub fn boolean_intersection(a: &Path, b: &Path, tolerance: Tolerance) -> CoreResult<Path> {
    boolean_paths(a, b, BooleanOperation::Intersection, tolerance)
}

pub fn boolean_difference(a: &Path, b: &Path, tolerance: Tolerance) -> CoreResult<Path> {
    boolean_paths(a, b, BooleanOperation::Difference, tolerance)
}

pub fn boolean_xor(a: &Path, b: &Path, tolerance: Tolerance) -> CoreResult<Path> {
    boolean_paths(a, b, BooleanOperation::Xor, tolerance)
}

pub fn normalize_self_intersections(
    path: &Path,
    fill_rule: FillRule,
    tolerance: Tolerance,
) -> CoreResult<Path> {
    validate_boolean_input(path)?;

    if path.is_empty() {
        return Ok(Path::new());
    }

    let segments = collect_segments(path, tolerance);
    let mut splits = vec![vec![0.0, 1.0]; segments.len()];

    for (index, working) in segments.iter().enumerate() {
        if let Segment::Cubic(curve) = working.segment {
            if let Some((first, second)) = cubic_self_intersection_parameters(curve, tolerance) {
                add_parameter(&mut splits[index], first, tolerance);
                add_parameter(&mut splits[index], second, tolerance);
            }
        }
    }

    let segment_bounds: Vec<_> = segments
        .iter()
        .map(|working| working.segment.bounds())
        .collect();
    for (index_a, index_b) in spatial_self_candidate_pairs(&segment_bounds, tolerance.absolute)? {
        let intersections = intersect_segments(
            segments[index_a].segment,
            segments[index_b].segment,
            tolerance,
        )?;

        for intersection in intersections.intersections {
            match intersection {
                Intersection::Point(point) => {
                    add_parameter(&mut splits[index_a], point.parameter_a, tolerance);
                    add_parameter(&mut splits[index_b], point.parameter_b, tolerance);
                }
                Intersection::Overlap(overlap) => {
                    add_parameter(&mut splits[index_a], overlap.range_a.min, tolerance);
                    add_parameter(&mut splits[index_a], overlap.range_a.max, tolerance);
                    add_parameter(&mut splits[index_b], overlap.range_b.min, tolerance);
                    add_parameter(&mut splits[index_b], overlap.range_b.max, tolerance);
                }
            }
        }
    }

    normalize_split_parameters(&mut splits, tolerance);

    let fill_index = PathSpatialIndex::build(path, tolerance)?;
    let mut fragments = Vec::new();
    for (working, parameters) in segments.iter().zip(&splits) {
        for pair in parameters.windows(2) {
            let t0 = pair[0];
            let t1 = pair[1];

            if t1 - t0 <= parameter_tolerance(tolerance) {
                continue;
            }

            let fragment = segment_subrange(working.segment, t0, t1);
            if fragment.start().distance_to(fragment.end()) <= tolerance.absolute
                && fragment.bounds().width() <= tolerance.absolute
                && fragment.bounds().height() <= tolerance.absolute
            {
                continue;
            }

            if let Some(oriented) =
                classify_single_path_boundary(fragment, &fill_index, fill_rule, tolerance)?
            {
                fragments.push(BoundaryFragment { segment: oriented });
            }
        }
    }

    deduplicate_fragments(&mut fragments, tolerance);
    stitch_normalized_fragments(fragments, tolerance)
}

fn cubic_self_intersection_parameters(
    curve: CubicBezier,
    tolerance: Tolerance,
) -> Option<(Scalar, Scalar)> {
    let a = Vector2::new(
        -curve.p0.x + 3.0 * curve.p1.x - 3.0 * curve.p2.x + curve.p3.x,
        -curve.p0.y + 3.0 * curve.p1.y - 3.0 * curve.p2.y + curve.p3.y,
    );
    let b = Vector2::new(
        3.0 * curve.p0.x - 6.0 * curve.p1.x + 3.0 * curve.p2.x,
        3.0 * curve.p0.y - 6.0 * curve.p1.y + 3.0 * curve.p2.y,
    );
    let c = Vector2::new(
        -3.0 * curve.p0.x + 3.0 * curve.p1.x,
        -3.0 * curve.p0.y + 3.0 * curve.p1.y,
    );

    let determinant = a.cross(b);
    let coefficient_scale = a.length().max(b.length()).max(c.length()).max(1.0);
    let determinant_tolerance =
        (tolerance.absolute + tolerance.relative * coefficient_scale) * coefficient_scale * 32.0;

    if !determinant.is_finite() || determinant.abs() <= determinant_tolerance {
        return None;
    }

    let w = b.cross(c) / determinant;
    let sum = c.cross(a) / determinant;
    let discriminant = 4.0 * w - 3.0 * sum * sum;
    let parameter_epsilon = parameter_tolerance(tolerance);

    if !w.is_finite()
        || !sum.is_finite()
        || !discriminant.is_finite()
        || discriminant <= parameter_epsilon * parameter_epsilon
    {
        return None;
    }

    let root = discriminant.sqrt();
    let first = (sum - root) * 0.5;
    let second = (sum + root) * 0.5;

    if first < -parameter_epsilon
        || second > 1.0 + parameter_epsilon
        || second - first <= parameter_epsilon
    {
        return None;
    }

    let first = snap_parameter(first.clamp(0.0, 1.0), tolerance);
    let second = snap_parameter(second.clamp(0.0, 1.0), tolerance);

    if first == second || (first == 0.0 && second == 1.0) {
        None
    } else {
        Some((first, second))
    }
}

fn classify_single_path_boundary(
    fragment: Segment,
    path: &PathSpatialIndex,
    fill_rule: FillRule,
    tolerance: Tolerance,
) -> CoreResult<Option<Segment>> {
    let midpoint = fragment.point_at(0.5);
    let tangent = stable_tangent(fragment, tolerance)?;
    let normal = tangent.perpendicular();
    let probe = probe_distance(fragment, midpoint, tolerance);

    let Some((left, right)) = classify_sides(path, fill_rule, midpoint, normal, probe)? else {
        return Err(CoreError::ToleranceNotMet);
    };

    eprintln!(
        "NORM_FRAGMENT start=({:.9},{:.9}) end=({:.9},{:.9}) mid=({:.9},{:.9}) sides=({left},{right})",
        fragment.start().x,
        fragment.start().y,
        fragment.end().x,
        fragment.end().y,
        midpoint.x,
        midpoint.y,
    );

    if left == right {
        Ok(None)
    } else if left {
        Ok(Some(fragment))
    } else {
        Ok(Some(fragment.reversed()))
    }
}

fn stitch_normalized_fragments(
    mut fragments: Vec<BoundaryFragment>,
    tolerance: Tolerance,
) -> CoreResult<Path> {
    if fragments.is_empty() {
        return Ok(Path::new());
    }

    let mut builder = PathBuilder::with_tolerance(tolerance);
    let maximum_steps = fragments.len() + 1;

    while !fragments.is_empty() {
        let first = fragments.swap_remove(0);
        let start = first.segment.start();
        let mut current = first.segment.end();
        let mut chain = vec![first.segment];
        let mut steps = 0usize;

        while !current.almost_eq(start, tolerance) {
            steps += 1;
            if steps > maximum_steps {
                return Err(CoreError::IterationLimit);
            }

            let previous = *chain.last().expect("chain is never empty");
            let Some(index) = face_continuation(&fragments, current, previous, tolerance) else {
                eprintln!(
                    "NORM_STITCH_MISS current=({:.9},{:.9}) start=({:.9},{:.9}) remaining={}",
                    current.x,
                    current.y,
                    start.x,
                    start.y,
                    fragments.len(),
                );
                for fragment in &fragments {
                    eprintln!(
                        "NORM_REMAIN start=({:.9},{:.9}) end=({:.9},{:.9})",
                        fragment.segment.start().x,
                        fragment.segment.start().y,
                        fragment.segment.end().x,
                        fragment.segment.end().y,
                    );
                }
                return Err(CoreError::NonConvergent);
            };

            let next = fragments.swap_remove(index);
            current = next.segment.end();
            chain.push(next.segment);
        }

        builder.move_to(start)?;
        for segment in chain {
            append_segment(&mut builder, segment)?;
        }
        builder.close()?;
    }

    builder.finish()
}

fn face_continuation(
    fragments: &[BoundaryFragment],
    current: Point2,
    previous: Segment,
    tolerance: Tolerance,
) -> Option<usize> {
    let mut candidates: Vec<(usize, Scalar)> = fragments
        .iter()
        .enumerate()
        .filter_map(|(index, fragment)| {
            let distance = current.distance_to(fragment.segment.start());
            if distance <= endpoint_tolerance(current, fragment.segment.start(), tolerance) {
                Some((index, distance))
            } else {
                None
            }
        })
        .collect();

    if candidates.len() <= 1 {
        return candidates.first().map(|candidate| candidate.0);
    }

    let incoming = tangent_at_end(previous, tolerance).ok()?;
    let reverse = -incoming;

    candidates.sort_by(|left, right| {
        let turn_left = clockwise_turn(
            reverse,
            tangent_at_start(fragments[left.0].segment, tolerance).ok(),
        );
        let turn_right = clockwise_turn(
            reverse,
            tangent_at_start(fragments[right.0].segment, tolerance).ok(),
        );

        turn_left
            .total_cmp(&turn_right)
            .then_with(|| left.1.total_cmp(&right.1))
            .then_with(|| left.0.cmp(&right.0))
    });

    candidates.first().map(|candidate| candidate.0)
}

fn clockwise_turn(reference: Vector2, candidate: Option<Vector2>) -> Scalar {
    let Some(candidate) = candidate else {
        return Scalar::INFINITY;
    };

    let signed_ccw = reference.cross(candidate).atan2(reference.dot(candidate));
    (-signed_ccw).rem_euclid(core::f64::consts::TAU)
}

pub fn boolean_paths(
    a: &Path,
    b: &Path,
    operation: BooleanOperation,
    tolerance: Tolerance,
) -> CoreResult<Path> {
    boolean_paths_with_fill_rules(
        a,
        FillRule::NonZero,
        b,
        FillRule::NonZero,
        operation,
        tolerance,
    )
}

pub fn boolean_paths_with_fill_rules(
    a: &Path,
    fill_a: FillRule,
    b: &Path,
    fill_b: FillRule,
    operation: BooleanOperation,
    tolerance: Tolerance,
) -> CoreResult<Path> {
    validate_boolean_input(a)?;
    validate_boolean_input(b)?;

    let normalized_a = normalize_self_intersections(a, fill_a, tolerance)?;
    let normalized_b = normalize_self_intersections(b, fill_b, tolerance)?;
    let a = &normalized_a;
    let b = &normalized_b;

    if a.is_empty() || b.is_empty() {
        return boolean_with_empty(a, b, operation);
    }

    let fill_a = FillRule::NonZero;
    let fill_b = FillRule::NonZero;
    let segments_a = collect_segments(a, tolerance);
    let segments_b = collect_segments(b, tolerance);
    let mut splits_a = vec![vec![0.0, 1.0]; segments_a.len()];
    let mut splits_b = vec![vec![0.0, 1.0]; segments_b.len()];

    let bounds_a: Vec<_> = segments_a
        .iter()
        .map(|working| working.segment.bounds())
        .collect();
    let bounds_b: Vec<_> = segments_b
        .iter()
        .map(|working| working.segment.bounds())
        .collect();

    for (index_a, index_b) in
        spatial_cross_candidate_pairs(&bounds_a, &bounds_b, tolerance.absolute)?
    {
        let intersections = intersect_segments(
            segments_a[index_a].segment,
            segments_b[index_b].segment,
            tolerance,
        )?;

        for intersection in intersections.intersections {
            match intersection {
                Intersection::Point(point) => {
                    add_parameter(&mut splits_a[index_a], point.parameter_a, tolerance);
                    add_parameter(&mut splits_b[index_b], point.parameter_b, tolerance);
                }
                Intersection::Overlap(overlap) => {
                    add_parameter(&mut splits_a[index_a], overlap.range_a.min, tolerance);
                    add_parameter(&mut splits_a[index_a], overlap.range_a.max, tolerance);
                    add_parameter(&mut splits_b[index_b], overlap.range_b.min, tolerance);
                    add_parameter(&mut splits_b[index_b], overlap.range_b.max, tolerance);
                }
            }
        }
    }

    normalize_split_parameters(&mut splits_a, tolerance);
    normalize_split_parameters(&mut splits_b, tolerance);

    let index_a = PathSpatialIndex::build(a, tolerance)?;
    let index_b = PathSpatialIndex::build(b, tolerance)?;
    let context = BooleanContext {
        a: &index_a,
        fill_a,
        b: &index_b,
        fill_b,
        operation,
        tolerance,
    };

    let mut fragments = Vec::new();
    append_boolean_fragments(&mut fragments, &segments_a, &splits_a, context)?;
    append_boolean_fragments(&mut fragments, &segments_b, &splits_b, context)?;

    deduplicate_fragments(&mut fragments, tolerance);
    stitch_fragments(fragments, tolerance)
}

fn validate_boolean_input(path: &Path) -> CoreResult<()> {
    if path.subpaths().iter().any(|subpath| !subpath.is_closed()) {
        return Err(CoreError::UnsupportedCase);
    }

    Ok(())
}

fn boolean_with_empty(a: &Path, b: &Path, operation: BooleanOperation) -> CoreResult<Path> {
    Ok(match operation {
        BooleanOperation::Union | BooleanOperation::Xor => {
            if a.is_empty() {
                b.clone()
            } else {
                a.clone()
            }
        }
        BooleanOperation::Intersection => Path::new(),
        BooleanOperation::Difference => {
            if a.is_empty() {
                Path::new()
            } else {
                a.clone()
            }
        }
    })
}

fn collect_segments(path: &Path, tolerance: Tolerance) -> Vec<WorkingSegment> {
    let mut result = Vec::new();

    for subpath in path.subpaths() {
        for &segment in subpath.segments() {
            result.push(WorkingSegment { segment });
        }

        if subpath.is_closed() && !subpath.end().almost_eq(subpath.start(), tolerance) {
            result.push(WorkingSegment {
                segment: Segment::Line(LineSegment::new(subpath.end(), subpath.start())),
            });
        }
    }

    result
}

fn add_parameter(parameters: &mut Vec<Scalar>, parameter: Scalar, tolerance: Tolerance) {
    if !parameter.is_finite() {
        return;
    }

    let parameter = snap_parameter(parameter.clamp(0.0, 1.0), tolerance);
    if parameters
        .iter()
        .all(|existing| (existing - parameter).abs() > parameter_tolerance(tolerance))
    {
        parameters.push(parameter);
    }
}

fn normalize_split_parameters(parameters: &mut [Vec<Scalar>], tolerance: Tolerance) {
    let epsilon = parameter_tolerance(tolerance);

    for values in parameters {
        values.sort_by(Scalar::total_cmp);
        values.dedup_by(|left, right| (*left - *right).abs() <= epsilon);

        if values.first().is_none_or(|value| *value != 0.0) {
            values.insert(0, 0.0);
        }
        if values.last().is_none_or(|value| *value != 1.0) {
            values.push(1.0);
        }
    }
}

fn append_boolean_fragments(
    output: &mut Vec<BoundaryFragment>,
    segments: &[WorkingSegment],
    splits: &[Vec<Scalar>],
    context: BooleanContext<'_>,
) -> CoreResult<()> {
    let tolerance = context.tolerance;
    for (working, parameters) in segments.iter().zip(splits) {
        for pair in parameters.windows(2) {
            let t0 = pair[0];
            let t1 = pair[1];

            if t1 - t0 <= parameter_tolerance(tolerance) {
                continue;
            }

            let fragment = segment_subrange(working.segment, t0, t1);
            if fragment.start().distance_to(fragment.end()) <= tolerance.absolute
                && fragment.bounds().width() <= tolerance.absolute
                && fragment.bounds().height() <= tolerance.absolute
            {
                continue;
            }

            if let Some(oriented) = classify_fragment_boundary(fragment, context)? {
                output.push(BoundaryFragment { segment: oriented });
            }
        }
    }

    Ok(())
}

fn classify_fragment_boundary(
    fragment: Segment,
    context: BooleanContext<'_>,
) -> CoreResult<Option<Segment>> {
    let tolerance = context.tolerance;
    let midpoint = fragment.point_at(0.5);
    let tangent = stable_tangent(fragment, tolerance)?;
    let normal = tangent.perpendicular();
    let probe = probe_distance(fragment, midpoint, tolerance);

    let Some((left_a, right_a)) =
        classify_sides(context.a, context.fill_a, midpoint, normal, probe)?
    else {
        return Err(CoreError::ToleranceNotMet);
    };
    let Some((left_b, right_b)) =
        classify_sides(context.b, context.fill_b, midpoint, normal, probe)?
    else {
        return Err(CoreError::ToleranceNotMet);
    };

    let left = operation_value(context.operation, left_a, left_b);
    let right = operation_value(context.operation, right_a, right_b);

    if left == right {
        Ok(None)
    } else if left {
        Ok(Some(fragment))
    } else {
        Ok(Some(fragment.reversed()))
    }
}

fn classify_sides(
    path: &PathSpatialIndex,
    fill_rule: FillRule,
    midpoint: Point2,
    normal: Vector2,
    base_probe: Scalar,
) -> CoreResult<Option<(bool, bool)>> {
    let mut distance = base_probe;

    for _ in 0..MAX_PROBE_ATTEMPTS {
        let left = path.classify_point(midpoint + normal * distance, fill_rule)?;
        let right = path.classify_point(midpoint - normal * distance, fill_rule)?;

        if left != PointClassification::Boundary && right != PointClassification::Boundary {
            return Ok(Some((
                left == PointClassification::Inside,
                right == PointClassification::Inside,
            )));
        }

        distance *= 2.0;
    }

    Ok(None)
}

fn stable_tangent(fragment: Segment, tolerance: Tolerance) -> CoreResult<Vector2> {
    let derivative = match fragment {
        Segment::Line(line) => line.direction(),
        Segment::Quadratic(curve) => curve.derivative_at(0.5),
        Segment::Cubic(curve) => curve.derivative_at(0.5),
        Segment::Arc(arc) => arc.derivative_at(0.5),
    };

    derivative
        .normalized(tolerance)
        .or_else(|_| (fragment.end() - fragment.start()).normalized(tolerance))
}

fn probe_distance(fragment: Segment, midpoint: Point2, tolerance: Tolerance) -> Scalar {
    let bounds = fragment.bounds();
    let local_scale = bounds
        .width()
        .max(bounds.height())
        .max(fragment.start().distance_to(fragment.end()))
        .max(1.0e-12);
    let coordinate_scale = midpoint.x.abs().max(midpoint.y.abs()).max(1.0);
    let numerical = (tolerance.absolute + tolerance.relative * coordinate_scale) * 32.0;
    let flatten_probe = tolerance.flatness.min(local_scale * 1.0e-3) * 4.0;
    let desired = numerical.max(flatten_probe).max(1.0e-12);
    let cap = (local_scale * 0.1).max(numerical);

    desired.min(cap)
}

fn deduplicate_fragments(fragments: &mut Vec<BoundaryFragment>, tolerance: Tolerance) {
    let mut unique: Vec<BoundaryFragment> = Vec::with_capacity(fragments.len());

    for fragment in fragments.drain(..) {
        if unique
            .iter()
            .all(|existing| !same_directed_geometry(existing.segment, fragment.segment, tolerance))
        {
            unique.push(fragment);
        }
    }

    *fragments = unique;
}

fn same_directed_geometry(a: Segment, b: Segment, tolerance: Tolerance) -> bool {
    let geometry_tolerance = geometry_match_tolerance(a, b, tolerance);

    [0.0, 0.25, 0.5, 0.75, 1.0]
        .into_iter()
        .all(|t| a.point_at(t).distance_to(b.point_at(t)) <= geometry_tolerance)
}

fn geometry_match_tolerance(a: Segment, b: Segment, tolerance: Tolerance) -> Scalar {
    let bounds_a = a.bounds();
    let bounds_b = b.bounds();
    let coordinate_scale = [
        a.start().x.abs(),
        a.start().y.abs(),
        a.end().x.abs(),
        a.end().y.abs(),
        b.start().x.abs(),
        b.start().y.abs(),
        b.end().x.abs(),
        b.end().y.abs(),
        bounds_a.width(),
        bounds_a.height(),
        bounds_b.width(),
        bounds_b.height(),
    ]
    .into_iter()
    .fold(1.0, Scalar::max);

    (tolerance.absolute + tolerance.relative * coordinate_scale) * 16.0
}

fn stitch_fragments(
    mut fragments: Vec<BoundaryFragment>,
    tolerance: Tolerance,
) -> CoreResult<Path> {
    if fragments.is_empty() {
        return Ok(Path::new());
    }

    let mut builder = PathBuilder::with_tolerance(tolerance);
    let maximum_steps = fragments.len() + 1;

    while !fragments.is_empty() {
        let first = fragments.swap_remove(0);
        let start = first.segment.start();
        let mut current = first.segment.end();
        let mut chain = vec![first.segment];
        let mut steps = 0usize;

        while !current.almost_eq(start, tolerance) {
            steps += 1;
            if steps > maximum_steps {
                return Err(CoreError::IterationLimit);
            }

            let Some(index) =
                best_continuation(&fragments, current, chain.last().copied(), tolerance)
            else {
                return Err(CoreError::NonConvergent);
            };

            let next = fragments.swap_remove(index);
            current = next.segment.end();
            chain.push(next.segment);
        }

        builder.move_to(start)?;
        for segment in chain {
            append_segment(&mut builder, segment)?;
        }
        builder.close()?;
    }

    builder.finish()
}

fn best_continuation(
    fragments: &[BoundaryFragment],
    current: Point2,
    previous: Option<Segment>,
    tolerance: Tolerance,
) -> Option<usize> {
    let mut candidates: Vec<(usize, Scalar)> = fragments
        .iter()
        .enumerate()
        .filter_map(|(index, fragment)| {
            let distance = current.distance_to(fragment.segment.start());
            if distance <= endpoint_tolerance(current, fragment.segment.start(), tolerance) {
                Some((index, distance))
            } else {
                None
            }
        })
        .collect();

    if candidates.len() <= 1 {
        return candidates.first().map(|candidate| candidate.0);
    }

    let previous_direction = previous.and_then(|segment| tangent_at_end(segment, tolerance).ok());

    candidates.sort_by(|left, right| {
        let score_left =
            continuation_score(previous_direction, fragments[left.0].segment, tolerance);
        let score_right =
            continuation_score(previous_direction, fragments[right.0].segment, tolerance);

        score_right
            .total_cmp(&score_left)
            .then_with(|| left.1.total_cmp(&right.1))
    });

    candidates.first().map(|candidate| candidate.0)
}

fn continuation_score(
    previous_direction: Option<Vector2>,
    next: Segment,
    tolerance: Tolerance,
) -> Scalar {
    let Some(previous) = previous_direction else {
        return 0.0;
    };

    tangent_at_start(next, tolerance).map_or(-2.0, |direction| previous.dot(direction))
}

fn tangent_at_start(segment: Segment, tolerance: Tolerance) -> CoreResult<Vector2> {
    match segment {
        Segment::Line(line) => line.direction().normalized(tolerance),
        Segment::Quadratic(curve) => curve.derivative_at(0.0).normalized(tolerance),
        Segment::Cubic(curve) => curve.derivative_at(0.0).normalized(tolerance),
        Segment::Arc(arc) => arc.derivative_at(0.0).normalized(tolerance),
    }
}

fn tangent_at_end(segment: Segment, tolerance: Tolerance) -> CoreResult<Vector2> {
    match segment {
        Segment::Line(line) => line.direction().normalized(tolerance),
        Segment::Quadratic(curve) => curve.derivative_at(1.0).normalized(tolerance),
        Segment::Cubic(curve) => curve.derivative_at(1.0).normalized(tolerance),
        Segment::Arc(arc) => arc.derivative_at(1.0).normalized(tolerance),
    }
}

fn endpoint_tolerance(a: Point2, b: Point2, tolerance: Tolerance) -> Scalar {
    let scale =
        a.x.abs()
            .max(a.y.abs())
            .max(b.x.abs())
            .max(b.y.abs())
            .max(1.0);

    (tolerance.absolute + tolerance.relative * scale) * 32.0
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

fn segment_subrange(segment: Segment, t0: Scalar, t1: Scalar) -> Segment {
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

fn parameter_tolerance(tolerance: Tolerance) -> Scalar {
    PARAMETER_EPSILON.max(tolerance.relative * 16.0)
}

fn snap_parameter(parameter: Scalar, tolerance: Tolerance) -> Scalar {
    let epsilon = parameter_tolerance(tolerance);

    if parameter <= epsilon {
        0.0
    } else if parameter >= 1.0 - epsilon {
        1.0
    } else {
        parameter
    }
}
