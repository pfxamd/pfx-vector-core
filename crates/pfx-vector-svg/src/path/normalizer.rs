use pfx_vector_core::{
    Angle, CoreError, CoreResult, EllipticalArc, Path, PathBuilder, Point2, Vector2,
};

use super::SvgPathCommand;

fn resolve(base: Point2, x: f64, y: f64, relative: bool) -> Point2 {
    if relative {
        Point2::new(base.x + x, base.y + y)
    } else {
        Point2::new(x, y)
    }
}

pub fn svg_arc_to_center(
    start: Point2,
    end: Point2,
    rx: f64,
    ry: f64,
    rotation_deg: f64,
    large_arc: bool,
    sweep: bool,
) -> CoreResult<Option<EllipticalArc>> {
    let mut rx = rx.abs();
    let mut ry = ry.abs();

    if rx == 0.0 || ry == 0.0 || start == end {
        return Ok(None);
    }

    let phi = rotation_deg.to_radians();
    let (sin_phi, cos_phi) = phi.sin_cos();
    let dx = (start.x - end.x) * 0.5;
    let dy = (start.y - end.y) * 0.5;
    let x_prime = cos_phi * dx + sin_phi * dy;
    let y_prime = -sin_phi * dx + cos_phi * dy;

    let lambda = x_prime * x_prime / (rx * rx) + y_prime * y_prime / (ry * ry);
    if lambda > 1.0 {
        let scale = lambda.sqrt();
        rx *= scale;
        ry *= scale;
    }

    let numerator =
        (rx * rx * ry * ry - rx * rx * y_prime * y_prime - ry * ry * x_prime * x_prime).max(0.0);
    let denominator = rx * rx * y_prime * y_prime + ry * ry * x_prime * x_prime;
    let sign = if large_arc == sweep { -1.0 } else { 1.0 };
    let coefficient = if denominator == 0.0 {
        0.0
    } else {
        sign * (numerator / denominator).sqrt()
    };

    let cx_prime = coefficient * (rx * y_prime / ry);
    let cy_prime = coefficient * (-ry * x_prime / rx);
    let cx = cos_phi * cx_prime - sin_phi * cy_prime + (start.x + end.x) * 0.5;
    let cy = sin_phi * cx_prime + cos_phi * cy_prime + (start.y + end.y) * 0.5;

    let angle_between = |u: Vector2, v: Vector2| u.cross(v).atan2(u.dot(v));
    let u = Vector2::new((x_prime - cx_prime) / rx, (y_prime - cy_prime) / ry);
    let v = Vector2::new((-x_prime - cx_prime) / rx, (-y_prime - cy_prime) / ry);
    let theta1 = angle_between(Vector2::new(1.0, 0.0), u);
    let mut delta = angle_between(u, v);

    if !sweep && delta > 0.0 {
        delta -= core::f64::consts::TAU;
    } else if sweep && delta < 0.0 {
        delta += core::f64::consts::TAU;
    }

    Ok(Some(EllipticalArc::new(
        Point2::new(cx, cy),
        rx,
        ry,
        Angle::radians(phi),
        Angle::radians(theta1),
        Angle::radians(delta),
    )))
}

pub fn normalize_path_commands(commands: &[SvgPathCommand]) -> CoreResult<Path> {
    let mut builder = PathBuilder::new();
    let mut current = Point2::new(0.0, 0.0);
    let mut subpath_start = None;
    let mut last_cubic_control: Option<Point2> = None;
    let mut last_quadratic_control: Option<Point2> = None;

    for command in commands {
        match *command {
            SvgPathCommand::Move { relative, x, y } => {
                current = resolve(current, x, y, relative);
                builder.move_to(current)?;
                subpath_start = Some(current);
                last_cubic_control = None;
                last_quadratic_control = None;
            }
            SvgPathCommand::Line { relative, x, y } => {
                current = resolve(current, x, y, relative);
                builder.line_to(current)?;
                last_cubic_control = None;
                last_quadratic_control = None;
            }
            SvgPathCommand::Horizontal { relative, x } => {
                current = Point2::new(if relative { current.x + x } else { x }, current.y);
                builder.line_to(current)?;
                last_cubic_control = None;
                last_quadratic_control = None;
            }
            SvgPathCommand::Vertical { relative, y } => {
                current = Point2::new(current.x, if relative { current.y + y } else { y });
                builder.line_to(current)?;
                last_cubic_control = None;
                last_quadratic_control = None;
            }
            SvgPathCommand::Cubic {
                relative,
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                let control1 = resolve(current, x1, y1, relative);
                let control2 = resolve(current, x2, y2, relative);
                let point = resolve(current, x, y, relative);
                builder.cubic_to(control1, control2, point)?;
                current = point;
                last_cubic_control = Some(control2);
                last_quadratic_control = None;
            }
            SvgPathCommand::SmoothCubic {
                relative,
                x2,
                y2,
                x,
                y,
            } => {
                let control1 =
                    last_cubic_control.map_or(current, |previous| current + (current - previous));
                let control2 = resolve(current, x2, y2, relative);
                let point = resolve(current, x, y, relative);
                builder.cubic_to(control1, control2, point)?;
                current = point;
                last_cubic_control = Some(control2);
                last_quadratic_control = None;
            }
            SvgPathCommand::Quadratic {
                relative,
                x1,
                y1,
                x,
                y,
            } => {
                let control = resolve(current, x1, y1, relative);
                let point = resolve(current, x, y, relative);
                builder.quad_to(control, point)?;
                current = point;
                last_quadratic_control = Some(control);
                last_cubic_control = None;
            }
            SvgPathCommand::SmoothQuadratic { relative, x, y } => {
                let control = last_quadratic_control
                    .map_or(current, |previous| current + (current - previous));
                let point = resolve(current, x, y, relative);
                builder.quad_to(control, point)?;
                current = point;
                last_quadratic_control = Some(control);
                last_cubic_control = None;
            }
            SvgPathCommand::Arc {
                relative,
                rx,
                ry,
                rotation,
                large_arc,
                sweep,
                x,
                y,
            } => {
                let point = resolve(current, x, y, relative);

                if let Some(arc) =
                    svg_arc_to_center(current, point, rx, ry, rotation, large_arc, sweep)?
                {
                    builder.arc_to(arc)?;
                } else if current != point {
                    builder.line_to(point)?;
                }

                current = point;
                last_cubic_control = None;
                last_quadratic_control = None;
            }
            SvgPathCommand::Close => {
                builder.close()?;
                current = subpath_start.ok_or(CoreError::InvalidGeometry)?;
                subpath_start = None;
                last_cubic_control = None;
                last_quadratic_control = None;
            }
        }
    }

    builder.finish()
}
