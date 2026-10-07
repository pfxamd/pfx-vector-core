use super::measure::segment_parameter_at_length_with_total;
use crate::{
    Angle, Bounds, CoreError, CoreResult, EllipticalArc, LineSegment, Path, PathBuilder, Point2,
    Scalar, Segment, Subpath, Tolerance, flatten_path, segment_length,
};

const MAX_DASH_STEPS: usize = 262_144;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StrokeCap {
    Butt,
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StrokeJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StrokeStyle {
    pub width: Scalar,
    pub cap: StrokeCap,
    pub join: StrokeJoin,
    pub miter_limit: Scalar,
    pub dash_array: Vec<Scalar>,
    pub dash_offset: Scalar,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            width: 1.0,
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
            miter_limit: 4.0,
            dash_array: Vec::new(),
            dash_offset: 0.0,
        }
    }
}

impl StrokeStyle {
    pub fn validate(&self) -> CoreResult<()> {
        if !self.width.is_finite()
            || self.width < 0.0
            || !self.miter_limit.is_finite()
            || self.miter_limit < 0.0
            || !self.dash_offset.is_finite()
            || self.dash_array.iter().any(|v| !v.is_finite() || *v < 0.0)
        {
            return Err(CoreError::InvalidGeometry);
        }

        Ok(())
    }
}

#[must_use]
pub fn normalized_dash_pattern(style: &StrokeStyle) -> Vec<Scalar> {
    if style.dash_array.is_empty() || style.dash_array.iter().all(|v| *v == 0.0) {
        return Vec::new();
    }

    let mut pattern = style.dash_array.clone();
    if pattern.len() % 2 == 1 {
        pattern.extend(style.dash_array.iter().copied());
    }
    pattern
}

#[derive(Clone, Debug)]
struct DashFragment {
    segments: Vec<Segment>,
    closed: bool,
}

pub fn dash_path(path: &Path, style: &StrokeStyle, tolerance: Tolerance) -> CoreResult<Path> {
    style.validate()?;
    let pattern = normalized_dash_pattern(style);

    if path.is_empty() || pattern.is_empty() {
        return Ok(path.clone());
    }

    let mut fragments = Vec::new();
    for subpath in path.subpaths() {
        fragments.extend(dash_subpath(
            subpath,
            &pattern,
            style.dash_offset,
            tolerance,
        )?);
    }

    let mut builder = PathBuilder::with_tolerance(tolerance);
    for fragment in fragments {
        let Some(first) = fragment.segments.first().copied() else {
            continue;
        };

        builder.move_to(first.start())?;
        for segment in fragment.segments {
            append_segment(&mut builder, segment)?;
        }
        if fragment.closed {
            builder.close()?;
        }
    }

    builder.finish()
}

fn dash_subpath(
    subpath: &Subpath,
    pattern: &[Scalar],
    dash_offset: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Vec<DashFragment>> {
    let mut trace = subpath.segments().to_vec();
    if subpath.is_closed() && !subpath.end().almost_eq(subpath.start(), tolerance) {
        trace.push(Segment::Line(LineSegment::new(
            subpath.end(),
            subpath.start(),
        )));
    }

    if trace.is_empty() {
        return Ok(Vec::new());
    }

    let mut lengths = Vec::with_capacity(trace.len());
    let mut total = 0.0;
    for &segment in &trace {
        let length = segment_length(segment, tolerance)?;
        lengths.push(length);
        total += length;
    }

    if total == 0.0 {
        return Ok(Vec::new());
    }

    let intervals = dash_intervals(total, pattern, dash_offset)?;
    if intervals.is_empty() {
        return Ok(Vec::new());
    }

    let numerical = f64::EPSILON * total.abs().max(1.0) * 64.0;
    if subpath.is_closed()
        && intervals.len() == 1
        && intervals[0].0 <= numerical
        && total - intervals[0].1 <= numerical
    {
        return Ok(vec![DashFragment {
            segments: subpath.segments().to_vec(),
            closed: true,
        }]);
    }

    let mut fragment_segments = Vec::with_capacity(intervals.len());
    for &(start, end) in &intervals {
        fragment_segments.push(slice_trace_interval(
            &trace,
            &lengths,
            start,
            end,
            tolerance,
        )?);
    }

    let crosses_seam = subpath.is_closed()
        && intervals.len() > 1
        && intervals[0].0 <= numerical
        && total - intervals.last().expect("intervals is not empty").1 <= numerical
        && fragment_segments.len() == intervals.len();

    if crosses_seam {
        let first = fragment_segments.remove(0);
        if let Some(mut last) = fragment_segments.pop() {
            last.extend(first);
            fragment_segments.insert(0, last);
        }
    }

    Ok(fragment_segments
        .into_iter()
        .filter(|segments| !segments.is_empty())
        .map(|segments| DashFragment {
            segments,
            closed: false,
        })
        .collect())
}

fn dash_intervals(
    total: Scalar,
    pattern: &[Scalar],
    dash_offset: Scalar,
) -> CoreResult<Vec<(Scalar, Scalar)>> {
    let period: Scalar = pattern.iter().sum();
    if period == 0.0 {
        return Ok(vec![(0.0, total)]);
    }

    if !period.is_finite() {
        return Err(CoreError::InvalidGeometry);
    }

    let mut phase = dash_offset.rem_euclid(period);
    let mut index = 0usize;
    while index + 1 < pattern.len() && phase >= pattern[index] {
        phase -= pattern[index];
        index += 1;
    }

    let mut remaining = pattern[index] - phase;
    let mut position = 0.0;
    let mut intervals = Vec::new();
    let mut steps = 0usize;

    while position < total {
        while remaining == 0.0 {
            index = (index + 1) % pattern.len();
            remaining = pattern[index];
            steps += 1;
            if steps > MAX_DASH_STEPS {
                return Err(CoreError::IterationLimit);
            }
        }

        let step = remaining.min(total - position);
        if step <= 0.0 || position + step == position {
            return Err(CoreError::IterationLimit);
        }

        let end = position + step;
        if index % 2 == 0 {
            intervals.push((position, end));
            if intervals.len() > MAX_DASH_STEPS {
                return Err(CoreError::IterationLimit);
            }
        }

        position = end;
        remaining -= step;
        steps += 1;
        if steps > MAX_DASH_STEPS {
            return Err(CoreError::IterationLimit);
        }

        if remaining == 0.0 {
            index = (index + 1) % pattern.len();
            remaining = pattern[index];
        }
    }

    Ok(intervals)
}

fn slice_trace_interval(
    trace: &[Segment],
    lengths: &[Scalar],
    start: Scalar,
    end: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Vec<Segment>> {
    let mut output = Vec::new();
    let mut accumulated = 0.0;

    for (&segment, &length) in trace.iter().zip(lengths) {
        let segment_start = accumulated;
        let segment_end = accumulated + length;
        accumulated = segment_end;

        if length == 0.0 || end <= segment_start {
            if end <= segment_start {
                break;
            }
            continue;
        }
        if start >= segment_end {
            continue;
        }

        let local_start = (start - segment_start).clamp(0.0, length);
        let local_end = (end - segment_start).clamp(0.0, length);
        if local_end <= local_start {
            continue;
        }

        let t0 = if local_start == 0.0 {
            0.0
        } else {
            segment_parameter_at_length_with_total(segment, local_start, length, tolerance)?
        };
        let t1 = if local_end == length {
            1.0
        } else {
            segment_parameter_at_length_with_total(segment, local_end, length, tolerance)?
        };

        if t1 > t0 {
            output.push(slice_segment(segment, t0, t1));
        }
    }

    Ok(output)
}

fn slice_segment(segment: Segment, t0: Scalar, t1: Scalar) -> Segment {
    let t0 = t0.clamp(0.0, 1.0);
    let t1 = t1.clamp(t0, 1.0);

    match segment {
        Segment::Line(line) => Segment::Line(LineSegment::new(line.point_at(t0), line.point_at(t1))),
        Segment::Quadratic(curve) => Segment::Quadratic(curve.subcurve(t0, t1)),
        Segment::Cubic(curve) => {
            if t0 == 0.0 && t1 == 1.0 {
                return Segment::Cubic(curve);
            }
            let (_, right) = curve.split(t0);
            let local = if t0 == 1.0 {
                0.0
            } else {
                ((t1 - t0) / (1.0 - t0)).clamp(0.0, 1.0)
            };
            Segment::Cubic(right.split(local).0)
        }
        Segment::Arc(arc) => {
            let sweep = arc.sweep_angle.as_radians();
            Segment::Arc(EllipticalArc::new(
                arc.center,
                arc.radius_x,
                arc.radius_y,
                arc.rotation,
                Angle::radians(arc.start_angle.as_radians() + sweep * t0),
                Angle::radians(sweep * (t1 - t0)),
            ))
        }
    }
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

pub fn stroke_bounds(path: &Path, style: &StrokeStyle, tol: Tolerance) -> CoreResult<Bounds> {
    style.validate()?;
    if !normalized_dash_pattern(style).is_empty() {
        let dashed = dash_path(path, style, tol)?;
        let mut solid = style.clone();
        solid.dash_array.clear();
        solid.dash_offset = 0.0;
        return stroke_bounds(&dashed, &solid, tol);
    }

    let half = style.width * 0.5;
    let mut bounds = Bounds::Empty;
    for subpath in flatten_path(path, tol)? {
        for point in subpath.points {
            bounds = bounds
                .include(Point2::new(point.x - half, point.y - half))
                .include(Point2::new(point.x + half, point.y + half));
        }
    }
    Ok(bounds)
}

fn distance_to_segment(point: Point2, start: Point2, end: Point2) -> Scalar {
    let direction = end - start;
    let denominator = direction.dot(direction);
    if denominator == 0.0 {
        return point.distance_to(start);
    }

    let t = ((point - start).dot(direction) / denominator).clamp(0.0, 1.0);
    point.distance_to(start + direction * t)
}

pub fn stroke_contains_point(
    path: &Path,
    style: &StrokeStyle,
    point: Point2,
    tol: Tolerance,
) -> CoreResult<bool> {
    style.validate()?;
    if style.width == 0.0 {
        return Ok(false);
    }

    if !normalized_dash_pattern(style).is_empty() {
        let dashed = dash_path(path, style, tol)?;
        let mut solid = style.clone();
        solid.dash_array.clear();
        solid.dash_offset = 0.0;
        return stroke_contains_point(&dashed, &solid, point, tol);
    }

    let half = style.width * 0.5 + tol.absolute;
    for subpath in flatten_path(path, tol)? {
        for window in subpath.points.windows(2) {
            if distance_to_segment(point, window[0], window[1]) <= half {
                return Ok(true);
            }
        }
        if subpath.closed
            && subpath.points.len() > 2
            && distance_to_segment(
                point,
                *subpath.points.last().expect("closed subpath is not empty"),
                subpath.points[0],
            ) <= half
        {
            return Ok(true);
        }
    }

    Ok(false)
}
