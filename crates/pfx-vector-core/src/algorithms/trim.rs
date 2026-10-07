use super::measure::segment_parameter_at_length_with_total;
use crate::{
    CoreError, CoreResult, LineSegment, Path, PathBuilder, Scalar, Segment, Subpath, Tolerance,
    segment_length,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContourSliceMode {
    Clamp,
    Wrap,
}

pub fn contour_length(subpath: &Subpath, tolerance: Tolerance) -> CoreResult<Scalar> {
    let (_, _, total) = contour_trace_with_lengths(subpath, tolerance)?;
    Ok(total)
}

pub fn slice_contour(
    subpath: &Subpath,
    start_distance: Scalar,
    end_distance: Scalar,
    mode: ContourSliceMode,
    tolerance: Tolerance,
) -> CoreResult<Path> {
    if !start_distance.is_finite() || !end_distance.is_finite() {
        return Err(CoreError::InvalidNumber);
    }

    let (trace, lengths, total) = contour_trace_with_lengths(subpath, tolerance)?;
    if trace.is_empty() || total == 0.0 {
        return Ok(Path::new());
    }

    match mode {
        ContourSliceMode::Clamp => {
            let start = start_distance.clamp(0.0, total);
            let end = end_distance.clamp(0.0, total);

            if end < start {
                return Err(CoreError::InvalidGeometry);
            }
            if end == start {
                return Ok(Path::new());
            }
            if start == 0.0 && end == total {
                return Ok(Path::from_subpaths(vec![subpath.clone()]));
            }

            path_from_segments(
                slice_trace_interval(&trace, &lengths, start, end, tolerance)?,
                tolerance,
            )
        }
        ContourSliceMode::Wrap => {
            if !subpath.is_closed() {
                return Err(CoreError::UnsupportedCase);
            }

            let raw_span = end_distance - start_distance;
            if raw_span == 0.0 {
                return Ok(Path::new());
            }

            let numerical = tolerance
                .absolute
                .max(tolerance.relative * total.abs())
                .max(f64::EPSILON * total.abs().max(1.0) * 64.0);

            if raw_span.abs() >= total - numerical {
                return Ok(Path::from_subpaths(vec![subpath.clone()]));
            }

            let start = start_distance.rem_euclid(total);
            let span = if raw_span > 0.0 {
                raw_span
            } else {
                raw_span.rem_euclid(total)
            };

            if span == 0.0 {
                return Ok(Path::new());
            }
            if span >= total - numerical {
                return Ok(Path::from_subpaths(vec![subpath.clone()]));
            }

            let end = start + span;
            let mut segments = if end <= total {
                slice_trace_interval(&trace, &lengths, start, end, tolerance)?
            } else {
                let mut first = slice_trace_interval(&trace, &lengths, start, total, tolerance)?;
                first.extend(slice_trace_interval(
                    &trace,
                    &lengths,
                    0.0,
                    end - total,
                    tolerance,
                )?);
                first
            };

            if segments.len() > 1 {
                segments.retain(|segment| segment.start() != segment.end());
            }

            path_from_segments(segments, tolerance)
        }
    }
}

pub fn split_contour_at_length(
    subpath: &Subpath,
    distance: Scalar,
    tolerance: Tolerance,
) -> CoreResult<(Path, Path)> {
    if !distance.is_finite() {
        return Err(CoreError::InvalidNumber);
    }

    let total = contour_length(subpath, tolerance)?;
    let split = distance.clamp(0.0, total);
    Ok((
        slice_contour(subpath, 0.0, split, ContourSliceMode::Clamp, tolerance)?,
        slice_contour(
            subpath,
            split,
            total,
            ContourSliceMode::Clamp,
            tolerance,
        )?,
    ))
}

pub(crate) fn contour_trace_with_lengths(
    subpath: &Subpath,
    tolerance: Tolerance,
) -> CoreResult<(Vec<Segment>, Vec<Scalar>, Scalar)> {
    let mut trace = subpath.segments().to_vec();

    if subpath.is_closed() && !subpath.end().almost_eq(subpath.start(), tolerance) {
        trace.push(Segment::Line(LineSegment::new(
            subpath.end(),
            subpath.start(),
        )));
    }

    let mut lengths = Vec::with_capacity(trace.len());
    let mut total = 0.0;
    for &segment in &trace {
        let length = segment_length(segment, tolerance)?;
        lengths.push(length);
        total += length;
    }

    Ok((trace, lengths, total))
}

pub(crate) fn slice_trace_interval(
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

        if end <= segment_start {
            break;
        }
        if length == 0.0 || start >= segment_end {
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
        Segment::Line(line) => {
            Segment::Line(LineSegment::new(line.point_at(t0), line.point_at(t1)))
        }
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
            Segment::Arc(crate::EllipticalArc::new(
                arc.center,
                arc.radius_x,
                arc.radius_y,
                arc.rotation,
                crate::Angle::radians(arc.start_angle.as_radians() + sweep * t0),
                crate::Angle::radians(sweep * (t1 - t0)),
            ))
        }
    }
}

fn path_from_segments(segments: Vec<Segment>, tolerance: Tolerance) -> CoreResult<Path> {
    let Some(first) = segments.first().copied() else {
        return Ok(Path::new());
    };

    let mut builder = PathBuilder::with_tolerance(tolerance);
    builder.move_to(first.start())?;

    for segment in segments {
        append_segment(&mut builder, segment)?;
    }

    builder.finish()
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
