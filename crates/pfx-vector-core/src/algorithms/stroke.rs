use super::trim::{contour_trace_with_lengths, slice_trace_interval};
use crate::{
    Bounds, CoreError, CoreResult, FillRule, Path, PathBuilder, PathSpatialIndex, Point2, Scalar,
    Segment, Subpath, Tolerance, outline_path, path_bounds,
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
    let (trace, lengths, total) = contour_trace_with_lengths(subpath, tolerance)?;

    if trace.is_empty() {
        return Ok(Vec::new());
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
            &trace, &lengths, start, end, tolerance,
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

#[derive(Clone, Debug)]
pub struct StrokeHitIndex {
    outline: Path,
    spatial: PathSpatialIndex,
}

impl StrokeHitIndex {
    pub fn build(path: &Path, style: &StrokeStyle, tolerance: Tolerance) -> CoreResult<Self> {
        style.validate()?;
        let outline = outline_path(path, style, tolerance)?;
        let spatial = PathSpatialIndex::build(&outline, tolerance)?;
        Ok(Self { outline, spatial })
    }

    #[must_use]
    pub fn outline(&self) -> &Path {
        &self.outline
    }

    #[must_use]
    pub const fn bounds(&self) -> Bounds {
        self.spatial.bounds()
    }

    pub fn contains_point(&self, point: Point2) -> CoreResult<bool> {
        if self.outline.is_empty() {
            return Ok(false);
        }

        self.spatial.contains_point(point, FillRule::NonZero)
    }
}

pub fn stroke_bounds(path: &Path, style: &StrokeStyle, tolerance: Tolerance) -> CoreResult<Bounds> {
    style.validate()?;
    if style.width == 0.0 || path.is_empty() {
        return Ok(Bounds::Empty);
    }

    Ok(path_bounds(&outline_path(path, style, tolerance)?))
}

pub(crate) fn stroke_query_radius(style: &StrokeStyle, tolerance: Tolerance) -> Scalar {
    let half = style.width * 0.5;
    let cap_reach = match style.cap {
        StrokeCap::Butt | StrokeCap::Round => half,
        StrokeCap::Square => half * core::f64::consts::SQRT_2,
    };
    let join_reach = match style.join {
        StrokeJoin::Miter => half * style.miter_limit.max(core::f64::consts::SQRT_2),
        StrokeJoin::Round | StrokeJoin::Bevel => half * core::f64::consts::SQRT_2,
    };

    cap_reach.max(join_reach) + tolerance.absolute
}

pub fn stroke_contains_point(
    path: &Path,
    style: &StrokeStyle,
    point: Point2,
    tolerance: Tolerance,
) -> CoreResult<bool> {
    style.validate()?;
    if style.width == 0.0 || path.is_empty() {
        return Ok(false);
    }

    StrokeHitIndex::build(path, style, tolerance)?.contains_point(point)
}
