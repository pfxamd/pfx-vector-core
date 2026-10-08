use super::measure::{segment_length, segment_length_to_t};
use crate::{CoreError, CoreResult, Path, PathLocation, Point2, Scalar, Segment, Tolerance};

const MAX_TABLE_DEPTH: u32 = 18;
const INVERSE_ITERATIONS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq)]
struct ArcLengthSample {
    t: Scalar,
    distance: Scalar,
    speed: Scalar,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SegmentMeasureTable {
    segment: Segment,
    samples: Vec<ArcLengthSample>,
    total_length: Scalar,
    tolerance: Tolerance,
}

impl SegmentMeasureTable {
    pub fn build(segment: Segment, tolerance: Tolerance) -> CoreResult<Self> {
        let total_length = segment_length(segment, tolerance)?;
        if !total_length.is_finite() || total_length < 0.0 {
            return Err(CoreError::InvalidNumber);
        }

        let speed0 = segment.derivative_at(0.0).length();
        let speed1 = segment.derivative_at(1.0).length();
        if !speed0.is_finite() || !speed1.is_finite() {
            return Err(CoreError::InvalidNumber);
        }

        let mut samples = vec![ArcLengthSample {
            t: 0.0,
            distance: 0.0,
            speed: speed0,
        }];

        if total_length == 0.0 || matches!(segment, Segment::Line(_)) {
            samples.push(ArcLengthSample {
                t: 1.0,
                distance: total_length,
                speed: speed1,
            });
        } else {
            append_adaptive_samples(
                segment,
                tolerance,
                total_length,
                ArcLengthSample {
                    t: 0.0,
                    distance: 0.0,
                    speed: speed0,
                },
                ArcLengthSample {
                    t: 1.0,
                    distance: total_length,
                    speed: speed1,
                },
                MAX_TABLE_DEPTH,
                &mut samples,
            )?;
        }

        Ok(Self {
            segment,
            samples,
            total_length,
            tolerance,
        })
    }

    #[must_use]
    pub const fn segment(&self) -> Segment {
        self.segment
    }

    #[must_use]
    pub const fn total_length(&self) -> Scalar {
        self.total_length
    }

    #[must_use]
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    #[must_use]
    pub const fn tolerance(&self) -> Tolerance {
        self.tolerance
    }

    pub fn length_at_parameter(&self, t: Scalar) -> CoreResult<Scalar> {
        validate_unit_parameter(t)?;

        if t == 0.0 || self.total_length == 0.0 {
            return Ok(0.0);
        }
        if t == 1.0 {
            return Ok(self.total_length);
        }
        if matches!(self.segment, Segment::Line(_)) {
            return Ok(self.total_length * t);
        }

        let upper = self.samples.partition_point(|sample| sample.t < t);
        let right = self.samples[upper];
        let left = self.samples[upper - 1];
        Ok(monotone_hermite_distance(left, right, t))
    }

    pub fn parameter_at_length(&self, distance: Scalar) -> CoreResult<Scalar> {
        if !distance.is_finite() {
            return Err(CoreError::InvalidNumber);
        }

        if self.total_length == 0.0 || distance <= 0.0 {
            return Ok(0.0);
        }
        if distance >= self.total_length {
            return Ok(1.0);
        }
        if matches!(self.segment, Segment::Line(_)) {
            return Ok(distance / self.total_length);
        }

        let upper = self
            .samples
            .partition_point(|sample| sample.distance < distance);
        let right = self.samples[upper];
        let left = self.samples[upper - 1];

        let mut low = left.t;
        let mut high = right.t;
        for _ in 0..INVERSE_ITERATIONS {
            let mid = (low + high) * 0.5;
            if monotone_hermite_distance(left, right, mid) < distance {
                low = mid;
            } else {
                high = mid;
            }
        }

        Ok((low + high) * 0.5)
    }

    pub fn point_at_length(&self, distance: Scalar) -> CoreResult<Point2> {
        let t = self.parameter_at_length(distance)?;
        Ok(self.segment.point_at(t))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct MeasuredPathSegment {
    table: SegmentMeasureTable,
    start_distance: Scalar,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathMeasureIndex {
    segments: Vec<MeasuredPathSegment>,
    subpath_offsets: Vec<usize>,
    total_length: Scalar,
    tolerance: Tolerance,
}

impl PathMeasureIndex {
    pub fn build(path: &Path, tolerance: Tolerance) -> CoreResult<Self> {
        let mut segments = Vec::with_capacity(path.segment_count());
        let mut subpath_offsets = Vec::with_capacity(path.subpaths().len() + 1);
        let mut total_length = 0.0;

        for subpath in path.subpaths() {
            subpath_offsets.push(segments.len());
            for &segment in subpath.segments() {
                let table = SegmentMeasureTable::build(segment, tolerance)?;
                let start_distance = total_length;
                total_length += table.total_length();
                if !total_length.is_finite() {
                    return Err(CoreError::InvalidNumber);
                }
                segments.push(MeasuredPathSegment {
                    table,
                    start_distance,
                });
            }
        }
        subpath_offsets.push(segments.len());

        Ok(Self {
            segments,
            subpath_offsets,
            total_length,
            tolerance,
        })
    }

    #[must_use]
    pub const fn total_length(&self) -> Scalar {
        self.total_length
    }

    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    #[must_use]
    pub fn sample_count(&self) -> usize {
        self.segments
            .iter()
            .map(|entry| entry.table.sample_count())
            .sum()
    }

    #[must_use]
    pub const fn tolerance(&self) -> Tolerance {
        self.tolerance
    }

    pub fn segment_table(
        &self,
        subpath_index: usize,
        segment_index: usize,
    ) -> CoreResult<&SegmentMeasureTable> {
        let index = self.flat_segment_index(subpath_index, segment_index)?;
        Ok(&self.segments[index].table)
    }

    pub fn location_at_distance(&self, distance: Scalar) -> CoreResult<PathLocation> {
        if !distance.is_finite() {
            return Err(CoreError::InvalidNumber);
        }
        if self.segments.is_empty() {
            return Err(CoreError::DegenerateOperation);
        }

        let wanted = distance.clamp(0.0, self.total_length);
        let mut index = self
            .segments
            .partition_point(|entry| entry.start_distance + entry.table.total_length() < wanted);
        if index == self.segments.len() {
            index -= 1;
        }

        let entry = &self.segments[index];
        let local = (wanted - entry.start_distance).clamp(0.0, entry.table.total_length());
        let t = entry.table.parameter_at_length(local)?;
        let (subpath_index, segment_index) = self.source_address(index);

        Ok(PathLocation {
            subpath_index,
            segment_index,
            t,
            distance: wanted,
        })
    }

    pub fn distance_at_location(
        &self,
        subpath_index: usize,
        segment_index: usize,
        t: Scalar,
    ) -> CoreResult<Scalar> {
        let index = self.flat_segment_index(subpath_index, segment_index)?;
        let entry = &self.segments[index];
        Ok(entry.start_distance + entry.table.length_at_parameter(t)?)
    }

    pub fn point_at_length(&self, distance: Scalar) -> CoreResult<(Point2, PathLocation)> {
        let location = self.location_at_distance(distance)?;
        let segment = self
            .segment_table(location.subpath_index, location.segment_index)?
            .segment();
        Ok((segment.point_at(location.t), location))
    }

    fn flat_segment_index(&self, subpath_index: usize, segment_index: usize) -> CoreResult<usize> {
        let Some(next_subpath) = subpath_index.checked_add(1) else {
            return Err(CoreError::InvalidGeometry);
        };
        if next_subpath >= self.subpath_offsets.len() {
            return Err(CoreError::InvalidGeometry);
        }

        let start = self.subpath_offsets[subpath_index];
        let end = self.subpath_offsets[next_subpath];
        if segment_index >= end - start {
            return Err(CoreError::InvalidGeometry);
        }

        Ok(start + segment_index)
    }

    fn source_address(&self, flat_index: usize) -> (usize, usize) {
        let upper = self
            .subpath_offsets
            .partition_point(|offset| *offset <= flat_index);
        let subpath_index = upper.saturating_sub(1);
        (
            subpath_index,
            flat_index - self.subpath_offsets[subpath_index],
        )
    }
}

fn append_adaptive_samples(
    segment: Segment,
    tolerance: Tolerance,
    total_length: Scalar,
    left: ArcLengthSample,
    right: ArcLengthSample,
    depth: u32,
    output: &mut Vec<ArcLengthSample>,
) -> CoreResult<()> {
    let interval_length = right.distance - left.distance;
    let allowed_error = length_table_tolerance(segment, total_length, tolerance);

    if interval_length <= allowed_error {
        output.push(right);
        return Ok(());
    }

    let span = right.t - left.t;
    let q1_t = left.t + span * 0.25;
    let mid_t = left.t + span * 0.5;
    let q3_t = left.t + span * 0.75;

    let q1 = exact_sample(segment, left, q1_t, tolerance)?;
    let mid = exact_sample(segment, left, mid_t, tolerance)?;
    let q3 = exact_sample(segment, left, q3_t, tolerance)?;

    let maximum_error = [
        (q1.distance - monotone_hermite_distance(left, right, q1_t)).abs(),
        (mid.distance - monotone_hermite_distance(left, right, mid_t)).abs(),
        (q3.distance - monotone_hermite_distance(left, right, q3_t)).abs(),
    ]
    .into_iter()
    .fold(0.0_f64, Scalar::max);

    if maximum_error <= allowed_error {
        output.push(right);
        return Ok(());
    }
    if depth == 0 {
        return Err(CoreError::IterationLimit);
    }

    append_adaptive_samples(
        segment,
        tolerance,
        total_length,
        left,
        mid,
        depth - 1,
        output,
    )?;
    append_adaptive_samples(
        segment,
        tolerance,
        total_length,
        mid,
        right,
        depth - 1,
        output,
    )
}

fn exact_sample(
    segment: Segment,
    left: ArcLengthSample,
    t: Scalar,
    tolerance: Tolerance,
) -> CoreResult<ArcLengthSample> {
    // Integrate only the local interval; avoid recomputing the entire prefix
    // for each table sample while retaining original-curve derivatives.
    let distance = match segment {
        Segment::Line(_) => segment_length_to_t(segment, t, tolerance)?,
        Segment::Quadratic(_) | Segment::Cubic(_)
            if super::measure::line_like_bezier_length(segment, tolerance).is_some() =>
        {
            segment_length_to_t(segment, t, tolerance)?
        }
        _ => {
            // The table validator must use a more accurate reference than
            // the table's acceptance threshold; otherwise quadrature noise
            // can force endless refinement of already smooth intervals.
            let integration_tolerance = Tolerance::new(
                tolerance.absolute * 0.01,
                tolerance.relative * 0.01,
                tolerance.angular,
                tolerance.flatness,
            );
            crate::numeric::adaptive_simpson(
                |parameter| segment.derivative_at(parameter).length(),
                left.t,
                t,
                integration_tolerance,
            )?
        }
    };
    let distance = match segment {
        Segment::Line(_) => distance,
        Segment::Quadratic(_) | Segment::Cubic(_)
            if super::measure::line_like_bezier_length(segment, tolerance).is_some() => distance,
        _ => left.distance + distance,
    };
    let speed = segment.derivative_at(t).length();
    if !distance.is_finite() || !speed.is_finite() {
        return Err(CoreError::InvalidNumber);
    }

    Ok(ArcLengthSample { t, distance, speed })
}

fn length_table_tolerance(segment: Segment, total_length: Scalar, tolerance: Tolerance) -> Scalar {
    let model_tolerance = tolerance
        .absolute
        .max(tolerance.relative * total_length.abs())
        .max(1.0e-12)
        * 16.0;
    let numeric_floor = f64::EPSILON * segment_coordinate_scale(segment) * 32.0;
    model_tolerance.max(numeric_floor)
}

fn segment_coordinate_scale(segment: Segment) -> Scalar {
    let point_scale = |point: Point2| point.x.abs().max(point.y.abs());
    match segment {
        Segment::Line(line) => point_scale(line.start).max(point_scale(line.end)).max(1.0),
        Segment::Quadratic(curve) => point_scale(curve.p0)
            .max(point_scale(curve.p1))
            .max(point_scale(curve.p2))
            .max(1.0),
        Segment::Cubic(curve) => point_scale(curve.p0)
            .max(point_scale(curve.p1))
            .max(point_scale(curve.p2))
            .max(point_scale(curve.p3))
            .max(1.0),
        Segment::Arc(arc) => point_scale(arc.center)
            .max(arc.radius_x.abs())
            .max(arc.radius_y.abs())
            .max(1.0),
    }
}

fn monotone_hermite_distance(left: ArcLengthSample, right: ArcLengthSample, t: Scalar) -> Scalar {
    if t <= left.t {
        return left.distance;
    }
    if t >= right.t {
        return right.distance;
    }

    let dt = right.t - left.t;
    let delta = right.distance - left.distance;
    if dt <= 0.0 || delta <= 0.0 {
        return left.distance;
    }

    let secant = delta / dt;
    let mut d0 = left.speed.max(0.0);
    let mut d1 = right.speed.max(0.0);
    let alpha = d0 / secant;
    let beta = d1 / secant;
    let radius = alpha.hypot(beta);
    if radius > 3.0 {
        let scale = 3.0 / radius;
        d0 *= scale;
        d1 *= scale;
    }

    let u = (t - left.t) / dt;
    let u2 = u * u;
    let u3 = u2 * u;
    let h10 = u3 - 2.0 * u2 + u;
    let h01 = -2.0 * u3 + 3.0 * u2;
    let h11 = u3 - u2;

    left.distance + h10 * dt * d0 + h01 * delta + h11 * dt * d1
}

fn validate_unit_parameter(t: Scalar) -> CoreResult<()> {
    if !t.is_finite() {
        return Err(CoreError::InvalidNumber);
    }
    if !(0.0..=1.0).contains(&t) {
        return Err(CoreError::InvalidGeometry);
    }
    Ok(())
}
