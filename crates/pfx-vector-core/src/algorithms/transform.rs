use crate::{
    Angle, CoreError, CoreResult, EllipticalArc, Path, PathBuilder, Scalar, Segment, Tolerance,
    Transform2D, Vector2, flatten_path,
};

pub fn transform_path(path: &Path, t: Transform2D, tol: Tolerance) -> CoreResult<Path> {
    let mut b = PathBuilder::with_tolerance(tol);
    for sub in path.subpaths() {
        b.move_to(t.transform_point(sub.start()))?;
        for &seg in sub.segments() {
            match seg {
                Segment::Line(l) => {
                    b.line_to(t.transform_point(l.end))?;
                }
                Segment::Quadratic(q) => {
                    b.quad_to(t.transform_point(q.p1), t.transform_point(q.p2))?;
                }
                Segment::Cubic(c) => {
                    b.cubic_to(
                        t.transform_point(c.p1),
                        t.transform_point(c.p2),
                        t.transform_point(c.p3),
                    )?;
                }
                Segment::Arc(arc) => match transform_elliptical_arc(arc, t, tol) {
                    Ok(transformed) => {
                        b.arc_to(transformed)?;
                    }
                    Err(CoreError::SingularTransform) => {
                        append_flattened_transformed_arc(&mut b, arc, t, tol)?;
                    }
                    Err(error) => return Err(error),
                },
            }
        }
        if sub.is_closed() {
            b.close()?;
        }
    }
    b.finish()
}

pub fn transform_elliptical_arc(
    arc: EllipticalArc,
    transform: Transform2D,
    tolerance: Tolerance,
) -> CoreResult<EllipticalArc> {
    validate_arc_transform_inputs(arc, transform)?;

    let linear_scale = transform
        .a
        .abs()
        .max(transform.b.abs())
        .max(transform.c.abs())
        .max(transform.d.abs());
    let determinant = transform.determinant();
    if tolerance.nearly_zero(determinant, linear_scale * linear_scale) {
        return Err(CoreError::SingularTransform);
    }

    let rotation = arc.rotation.as_radians();
    let (sin_rotation, cos_rotation) = rotation.sin_cos();
    let axis_x = transform.transform_vector(Vector2::new(
        arc.radius_x * cos_rotation,
        arc.radius_x * sin_rotation,
    ));
    let axis_y = transform.transform_vector(Vector2::new(
        -arc.radius_y * sin_rotation,
        arc.radius_y * cos_rotation,
    ));

    let sxx = axis_x.x * axis_x.x + axis_y.x * axis_y.x;
    let sxy = axis_x.x * axis_x.y + axis_y.x * axis_y.y;
    let syy = axis_x.y * axis_x.y + axis_y.y * axis_y.y;
    if !sxx.is_finite() || !sxy.is_finite() || !syy.is_finite() {
        return Err(CoreError::InvalidNumber);
    }

    let trace = sxx + syy;
    let discriminant = (sxx - syy).hypot(2.0 * sxy);
    let radius_x_squared = 0.5 * (trace + discriminant);
    if !radius_x_squared.is_finite() || radius_x_squared <= 0.0 {
        return Err(CoreError::SingularTransform);
    }

    let shape_determinant = axis_x.x * axis_y.y - axis_x.y * axis_y.x;
    let radius_y_squared = shape_determinant * shape_determinant / radius_x_squared;
    if !radius_y_squared.is_finite() || radius_y_squared <= 0.0 {
        return Err(CoreError::SingularTransform);
    }

    let radius_x = radius_x_squared.sqrt();
    let radius_y = radius_y_squared.sqrt();
    if tolerance.nearly_zero(radius_y, radius_x) {
        return Err(CoreError::SingularTransform);
    }

    let target_rotation =
        if tolerance.nearly_zero(radius_x_squared - radius_y_squared, radius_x_squared) {
            0.0
        } else {
            0.5 * (2.0 * sxy).atan2(sxx - syy)
        };
    let (sin_target, cos_target) = target_rotation.sin_cos();

    let center = transform.transform_point(arc.center);
    let start_point = transform.transform_point(arc.point_at(0.0));
    let start_delta = start_point - center;
    let unit_x = (start_delta.x * cos_target + start_delta.y * sin_target) / radius_x;
    let unit_y = (-start_delta.x * sin_target + start_delta.y * cos_target) / radius_y;
    let unit_length = unit_x.hypot(unit_y);
    if !unit_length.is_finite() || tolerance.nearly_zero(unit_length, 1.0) {
        return Err(CoreError::InvalidGeometry);
    }

    let start_angle = (unit_y / unit_length).atan2(unit_x / unit_length);
    let sweep_sign = if determinant < 0.0 { -1.0 } else { 1.0 };

    Ok(EllipticalArc::new(
        center,
        radius_x,
        radius_y,
        Angle::radians(target_rotation),
        Angle::radians(start_angle),
        Angle::radians(arc.sweep_angle.as_radians() * sweep_sign),
    ))
}

fn validate_arc_transform_inputs(arc: EllipticalArc, transform: Transform2D) -> CoreResult<()> {
    let transform_is_finite = [
        transform.a,
        transform.b,
        transform.c,
        transform.d,
        transform.e,
        transform.f,
    ]
    .into_iter()
    .all(Scalar::is_finite);
    if !transform_is_finite {
        return Err(CoreError::InvalidNumber);
    }

    if !arc.center.is_finite()
        || !arc.radius_x.is_finite()
        || !arc.radius_y.is_finite()
        || !arc.rotation.as_radians().is_finite()
        || !arc.start_angle.as_radians().is_finite()
        || !arc.sweep_angle.as_radians().is_finite()
    {
        return Err(CoreError::InvalidNumber);
    }
    if arc.radius_x <= 0.0 || arc.radius_y <= 0.0 {
        return Err(CoreError::InvalidGeometry);
    }

    Ok(())
}

fn append_flattened_transformed_arc(
    builder: &mut PathBuilder,
    arc: EllipticalArc,
    transform: Transform2D,
    tolerance: Tolerance,
) -> CoreResult<()> {
    let mut source_builder = PathBuilder::with_tolerance(tolerance);
    source_builder.move_to(arc.point_at(0.0))?;
    source_builder.arc_to(arc)?;
    let source = source_builder.finish()?;
    let flattened = flatten_path(&source, tolerance)?;

    for point in flattened[0].points.iter().skip(1) {
        builder.line_to(transform.transform_point(*point))?;
    }

    Ok(())
}
