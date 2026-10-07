use crate::numeric::adaptive_simpson;
use crate::{
    CoreError, CoreResult, Path, PathLocation, Point2, Scalar, Segment, Tolerance, Vector2,
};

pub fn segment_length(segment: Segment, tolerance: Tolerance) -> CoreResult<Scalar> {
    match segment {
        Segment::Line(line) => Ok(line.length()),
        Segment::Quadratic(curve) => {
            adaptive_simpson(|t| curve.derivative_at(t).length(), 0.0, 1.0, tolerance)
        }
        Segment::Cubic(curve) => {
            adaptive_simpson(|t| curve.derivative_at(t).length(), 0.0, 1.0, tolerance)
        }
        Segment::Arc(arc) => {
            adaptive_simpson(|t| arc.derivative_at(t).length(), 0.0, 1.0, tolerance)
        }
    }
}

pub fn path_length(path: &Path, tolerance: Tolerance) -> CoreResult<Scalar> {
    let mut length = 0.0;

    for subpath in path.subpaths() {
        for &segment in subpath.segments() {
            length += segment_length(segment, tolerance)?;
        }
    }

    Ok(length)
}

pub(crate) fn segment_parameter_at_length_with_total(
    segment: Segment,
    target: Scalar,
    total: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Scalar> {
    if total == 0.0 {
        return Ok(0.0);
    }

    let mut low = 0.0;
    let mut high = 1.0;

    for _ in 0..48 {
        let mid = (low + high) * 0.5;
        let length = match segment {
            Segment::Line(_) => total * mid,
            Segment::Quadratic(curve) => {
                adaptive_simpson(|t| curve.derivative_at(t).length(), 0.0, mid, tolerance)?
            }
            Segment::Cubic(curve) => {
                adaptive_simpson(|t| curve.derivative_at(t).length(), 0.0, mid, tolerance)?
            }
            Segment::Arc(arc) => {
                adaptive_simpson(|t| arc.derivative_at(t).length(), 0.0, mid, tolerance)?
            }
        };

        if (length - target).abs() <= tolerance.absolute.max(tolerance.relative * total) {
            return Ok(mid);
        }

        if length < target {
            low = mid;
        } else {
            high = mid;
        }
    }

    Ok((low + high) * 0.5)
}

pub(crate) fn segment_length_to_t(
    segment: Segment,
    t: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Scalar> {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 {
        return Ok(0.0);
    }
    if t == 1.0 {
        return segment_length(segment, tolerance);
    }

    match segment {
        Segment::Line(line) => Ok(line.length() * t),
        Segment::Quadratic(curve) => {
            let (left, _) = curve.split(t);
            segment_length(Segment::Quadratic(left), tolerance)
        }
        Segment::Cubic(curve) => {
            let (left, _) = curve.split(t);
            segment_length(Segment::Cubic(left), tolerance)
        }
        Segment::Arc(arc) => {
            let partial = crate::EllipticalArc::new(
                arc.center,
                arc.radius_x,
                arc.radius_y,
                arc.rotation,
                arc.start_angle,
                crate::Angle::radians(arc.sweep_angle.as_radians() * t),
            );
            segment_length(Segment::Arc(partial), tolerance)
        }
    }
}

pub fn segment_parameter_at_length(
    segment: Segment,
    distance: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Scalar> {
    if !distance.is_finite() {
        return Err(CoreError::InvalidNumber);
    }

    let total = segment_length(segment, tolerance)?;
    segment_parameter_at_length_with_total(segment, distance.clamp(0.0, total), total, tolerance)
}

pub fn point_at_length(
    path: &Path,
    distance: Scalar,
    tolerance: Tolerance,
) -> CoreResult<(Point2, PathLocation)> {
    let total = path_length(path, tolerance)?;

    if path.is_empty() {
        return Err(CoreError::DegenerateOperation);
    }

    let wanted = distance.clamp(0.0, total);
    let mut accumulated = 0.0;
    let mut last = None;

    for (subpath_index, subpath) in path.subpaths().iter().enumerate() {
        for (segment_index, &segment) in subpath.segments().iter().enumerate() {
            let length = segment_length(segment, tolerance)?;
            last = Some((segment, subpath_index, segment_index, accumulated, length));

            if wanted <= accumulated + length || length == 0.0 {
                let local = (wanted - accumulated).clamp(0.0, length);
                let t = segment_parameter_at_length_with_total(segment, local, length, tolerance)?;
                return Ok((
                    segment.point_at(t),
                    PathLocation {
                        subpath_index,
                        segment_index,
                        t,
                        distance: wanted,
                    },
                ));
            }

            accumulated += length;
        }
    }

    let (segment, subpath_index, segment_index, start, _) =
        last.ok_or(CoreError::DegenerateOperation)?;

    Ok((
        segment.end(),
        PathLocation {
            subpath_index,
            segment_index,
            t: 1.0,
            distance: start,
        },
    ))
}

pub fn tangent_at_length(
    path: &Path,
    distance: Scalar,
    tolerance: Tolerance,
) -> CoreResult<Vector2> {
    let (_, location) = point_at_length(path, distance, tolerance)?;
    let segment = path.subpaths()[location.subpath_index].segments()[location.segment_index];

    match segment {
        Segment::Line(line) => line.direction().normalized(tolerance),
        Segment::Quadratic(curve) => curve.tangent_at(location.t, tolerance),
        Segment::Cubic(curve) => curve.tangent_at(location.t, tolerance),
        Segment::Arc(arc) => arc.derivative_at(location.t).normalized(tolerance),
    }
}
