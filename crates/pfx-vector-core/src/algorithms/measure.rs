use crate::numeric::{adaptive_simpson, solve_quadratic};
use crate::{
    CoreError, CoreResult, Path, PathLocation, Point2, Scalar, Segment, Tolerance, Vector2,
};

pub fn segment_length(segment: Segment, tolerance: Tolerance) -> CoreResult<Scalar> {
    match segment {
        Segment::Line(line) => Ok(line.length()),
        Segment::Quadratic(curve) => {
            if let Some(length) = line_like_bezier_length(Segment::Quadratic(curve), tolerance) {
                Ok(length)
            } else {
                adaptive_simpson(|t| curve.derivative_at(t).length(), 0.0, 1.0, tolerance)
            }
        }
        Segment::Cubic(curve) => {
            if let Some(length) = line_like_bezier_length(Segment::Cubic(curve), tolerance) {
                Ok(length)
            } else {
                adaptive_simpson(|t| curve.derivative_at(t).length(), 0.0, 1.0, tolerance)
            }
        }
        Segment::Arc(arc) => {
            adaptive_simpson(|t| arc.derivative_at(t).length(), 0.0, 1.0, tolerance)
        }
    }
}

fn line_like_bezier_length(segment: Segment, tolerance: Tolerance) -> Option<Scalar> {
    let controls = match segment {
        Segment::Quadratic(curve) => vec![curve.p0, curve.p1, curve.p2],
        Segment::Cubic(curve) => vec![curve.p0, curve.p1, curve.p2, curve.p3],
        _ => return None,
    };

    let mut origin = controls[0];
    let mut axis = Vector2::new(0.0, 0.0);
    let mut axis_length_squared = 0.0;

    for &left in &controls {
        for &right in &controls {
            let candidate = right - left;
            let candidate_length_squared = candidate.length_squared();
            if candidate_length_squared > axis_length_squared {
                origin = left;
                axis = candidate;
                axis_length_squared = candidate_length_squared;
            }
        }
    }

    if axis_length_squared <= tolerance.absolute * tolerance.absolute {
        return Some(0.0);
    }

    let axis_length = axis_length_squared.sqrt();
    let unit = axis / axis_length;
    let coordinate_scale = controls.iter().fold(1.0_f64, |scale, point| {
        scale.max(point.x.abs()).max(point.y.abs())
    });
    let geometric_tolerance =
        (tolerance.absolute + tolerance.relative * coordinate_scale) * 16.0;

    if controls
        .iter()
        .any(|point| axis.cross(*point - origin).abs() / axis_length > geometric_tolerance)
    {
        return None;
    }

    let project = |point: Point2| (point - origin).dot(unit);
    let mut parameters = vec![0.0, 1.0];

    match segment {
        Segment::Quadratic(curve) => {
            let p0 = project(curve.p0);
            let p1 = project(curve.p1);
            let p2 = project(curve.p2);
            let linear = 2.0 * (p1 - p0);
            let quadratic = p0 - 2.0 * p1 + p2;
            parameters.extend(solve_quadratic(0.0, 2.0 * quadratic, linear));
        }
        Segment::Cubic(curve) => {
            let p0 = project(curve.p0);
            let p1 = project(curve.p1);
            let p2 = project(curve.p2);
            let p3 = project(curve.p3);
            let linear = 3.0 * (p1 - p0);
            let quadratic = 3.0 * (p0 - 2.0 * p1 + p2);
            let cubic = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
            parameters.extend(solve_quadratic(
                3.0 * cubic,
                2.0 * quadratic,
                linear,
            ));
        }
        _ => unreachable!("line-like length is only used for Bézier segments"),
    }

    parameters.retain(|parameter| {
        parameter.is_finite() && *parameter > 0.0 && *parameter < 1.0
    });
    parameters.sort_by(Scalar::total_cmp);
    parameters.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-12);

    let projected_point = |t: Scalar| project(segment.point_at(t));
    let mut length = 0.0;
    for pair in parameters.windows(2) {
        length += (projected_point(pair[1]) - projected_point(pair[0])).abs();
    }

    Some(length)
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
