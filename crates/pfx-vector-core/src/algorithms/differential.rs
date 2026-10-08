use super::measure::{point_at_length, segment_length_to_t};
use crate::numeric::solve_quadratic;
use crate::{
    CoreError, CoreResult, Path, PathLocation, Point2, Scalar, Segment, Tolerance, Vector2,
    segment_length,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegmentFrame {
    pub point: Point2,
    pub tangent: Vector2,
    pub normal: Vector2,
    pub curvature: Scalar,
    pub speed: Scalar,
    pub t: Scalar,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathFrame {
    pub point: Point2,
    pub tangent: Vector2,
    pub normal: Vector2,
    pub curvature: Scalar,
    pub speed: Scalar,
    pub location: PathLocation,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathInflection {
    pub point: Point2,
    pub location: PathLocation,
}

pub fn segment_frame_at_t(
    segment: Segment,
    t: Scalar,
    tolerance: Tolerance,
) -> CoreResult<SegmentFrame> {
    validate_parameter(t)?;
    validate_segment(segment)?;

    let point = segment.point_at(t);
    let derivative = segment.derivative_at(t);
    let second_derivative = segment.second_derivative_at(t);
    if !point.is_finite()
        || !derivative.x.is_finite()
        || !derivative.y.is_finite()
        || !second_derivative.x.is_finite()
        || !second_derivative.y.is_finite()
    {
        return Err(CoreError::InvalidNumber);
    }

    let speed = derivative.length();
    let derivative_scale = derivative.x.abs().max(derivative.y.abs()).max(1.0);
    if tolerance.nearly_zero(speed, derivative_scale) {
        return Err(CoreError::DegenerateOperation);
    }

    let tangent = derivative / speed;
    let normal = tangent.perpendicular();
    let curvature = derivative.cross(second_derivative) / speed.powi(3);
    if !curvature.is_finite() {
        return Err(CoreError::InvalidNumber);
    }

    Ok(SegmentFrame {
        point,
        tangent,
        normal,
        curvature,
        speed,
        t,
    })
}

pub fn path_frame_at_length(
    path: &Path,
    distance: Scalar,
    tolerance: Tolerance,
) -> CoreResult<PathFrame> {
    if !distance.is_finite() {
        return Err(CoreError::InvalidNumber);
    }

    let (_, location) = point_at_length(path, distance, tolerance)?;
    let segment = path.subpaths()[location.subpath_index].segments()[location.segment_index];
    let frame = segment_frame_at_t(segment, location.t, tolerance)?;

    Ok(PathFrame {
        point: frame.point,
        tangent: frame.tangent,
        normal: frame.normal,
        curvature: frame.curvature,
        speed: frame.speed,
        location,
    })
}

pub fn segment_inflection_parameters(
    segment: Segment,
    tolerance: Tolerance,
) -> CoreResult<Vec<Scalar>> {
    validate_segment(segment)?;

    let Segment::Cubic(curve) = segment else {
        return Ok(Vec::new());
    };

    let a = Vector2::new(
        -curve.p0.x + 3.0 * curve.p1.x - 3.0 * curve.p2.x + curve.p3.x,
        -curve.p0.y + 3.0 * curve.p1.y - 3.0 * curve.p2.y + curve.p3.y,
    );
    let b = Vector2::new(
        3.0 * (curve.p0.x - 2.0 * curve.p1.x + curve.p2.x),
        3.0 * (curve.p0.y - 2.0 * curve.p1.y + curve.p2.y),
    );
    let c = (curve.p1 - curve.p0) * 3.0;

    let quadratic = -6.0 * a.cross(b);
    let linear = 6.0 * c.cross(a);
    let constant = 2.0 * c.cross(b);
    let mut roots = solve_quadratic(quadratic, linear, constant);

    roots.retain(|t| {
        if !t.is_finite() || *t <= 0.0 || *t >= 1.0 {
            return false;
        }

        let derivative = segment.derivative_at(*t);
        let speed = derivative.length();
        let scale = derivative.x.abs().max(derivative.y.abs()).max(1.0);
        if tolerance.nearly_zero(speed, scale) {
            return false;
        }

        let h = 1.0e-5_f64.min(*t * 0.5).min((1.0 - *t) * 0.5);
        if h <= 0.0 {
            return false;
        }

        let left = curvature_numerator(segment, *t - h);
        let right = curvature_numerator(segment, *t + h);
        left.is_finite() && right.is_finite() && left.signum() != right.signum()
    });

    roots.sort_by(Scalar::total_cmp);
    roots.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-12);
    Ok(roots)
}

pub fn path_inflections(path: &Path, tolerance: Tolerance) -> CoreResult<Vec<PathInflection>> {
    let mut output = Vec::new();
    let mut accumulated = 0.0;

    for (subpath_index, subpath) in path.subpaths().iter().enumerate() {
        for (segment_index, &segment) in subpath.segments().iter().enumerate() {
            for t in segment_inflection_parameters(segment, tolerance)? {
                let distance = accumulated + segment_length_to_t(segment, t, tolerance)?;
                output.push(PathInflection {
                    point: segment.point_at(t),
                    location: PathLocation {
                        subpath_index,
                        segment_index,
                        t,
                        distance,
                    },
                });
            }
            accumulated += segment_length(segment, tolerance)?;
        }
    }

    Ok(output)
}

fn curvature_numerator(segment: Segment, t: Scalar) -> Scalar {
    segment
        .derivative_at(t)
        .cross(segment.second_derivative_at(t))
}

fn validate_parameter(t: Scalar) -> CoreResult<()> {
    if !t.is_finite() {
        return Err(CoreError::InvalidNumber);
    }
    if !(0.0..=1.0).contains(&t) {
        return Err(CoreError::InvalidGeometry);
    }
    Ok(())
}

fn validate_segment(segment: Segment) -> CoreResult<()> {
    let valid = match segment {
        Segment::Line(line) => line.start.is_finite() && line.end.is_finite(),
        Segment::Quadratic(curve) => {
            curve.p0.is_finite() && curve.p1.is_finite() && curve.p2.is_finite()
        }
        Segment::Cubic(curve) => {
            curve.p0.is_finite()
                && curve.p1.is_finite()
                && curve.p2.is_finite()
                && curve.p3.is_finite()
        }
        Segment::Arc(arc) => {
            arc.center.is_finite()
                && arc.radius_x.is_finite()
                && arc.radius_y.is_finite()
                && arc.radius_x > 0.0
                && arc.radius_y > 0.0
                && arc.rotation.as_radians().is_finite()
                && arc.start_angle.as_radians().is_finite()
                && arc.sweep_angle.as_radians().is_finite()
        }
    };

    if valid {
        Ok(())
    } else {
        Err(CoreError::InvalidGeometry)
    }
}
