use crate::{
    Angle, Circle, CoreResult, Ellipse, EllipticalArc, Path, PathBuilder, Point2, Polygon,
    Polyline, Rect, RoundedRect, Tolerance,
};
pub fn rect_to_path(rect: Rect) -> CoreResult<Path> {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(rect.x, rect.y))?
        .line_to(Point2::new(rect.x + rect.width, rect.y))?
        .line_to(Point2::new(rect.x + rect.width, rect.y + rect.height))?
        .line_to(Point2::new(rect.x, rect.y + rect.height))?
        .close()?;
    builder.finish()
}
pub fn rounded_rect_to_path(rect: RoundedRect) -> CoreResult<Path> {
    let r = rect.rect;
    let rx = rect.radius_x;
    let ry = rect.radius_y;
    if rx == 0.0 || ry == 0.0 {
        return rect_to_path(r);
    }
    let mut b = PathBuilder::with_tolerance(Tolerance::default());
    b.move_to(Point2::new(r.x + rx, r.y))?
        .line_to(Point2::new(r.x + r.width - rx, r.y))?;
    let centers = [
        (
            Point2::new(r.x + r.width - rx, r.y + ry),
            -core::f64::consts::FRAC_PI_2,
        ),
        (Point2::new(r.x + r.width - rx, r.y + r.height - ry), 0.0),
        (
            Point2::new(r.x + rx, r.y + r.height - ry),
            core::f64::consts::FRAC_PI_2,
        ),
        (Point2::new(r.x + rx, r.y + ry), core::f64::consts::PI),
    ];
    let targets = [
        Point2::new(r.x + r.width, r.y + ry),
        Point2::new(r.x + r.width - rx, r.y + r.height),
        Point2::new(r.x, r.y + r.height - ry),
        Point2::new(r.x + rx, r.y),
    ];
    for (i, (center, start)) in centers.into_iter().enumerate() {
        let arc = EllipticalArc::new(
            center,
            rx,
            ry,
            Angle::radians(0.0),
            Angle::radians(start),
            Angle::radians(core::f64::consts::FRAC_PI_2),
        );
        b.arc_to(arc)?;
        let _ = targets[i];
        if i < 3 {
            b.line_to(targets[(i + 1) % 4])?;
        }
    }
    b.close()?;
    b.finish()
}
pub fn circle_to_path(circle: Circle) -> CoreResult<Path> {
    ellipse_to_path(Ellipse::new(
        circle.center,
        circle.radius,
        circle.radius,
        Angle::radians(0.0),
    )?)
}
pub fn ellipse_to_path(ellipse: Ellipse) -> CoreResult<Path> {
    let mut b = PathBuilder::with_tolerance(Tolerance::default());
    let start = EllipticalArc::new(
        ellipse.center,
        ellipse.radius_x,
        ellipse.radius_y,
        ellipse.rotation,
        Angle::radians(0.0),
        Angle::radians(core::f64::consts::TAU),
    );
    b.move_to(start.point_at(0.0))?.arc_to(start)?.close()?;
    b.finish()
}
pub fn polyline_to_path(p: &Polyline) -> CoreResult<Path> {
    let mut b = PathBuilder::new();
    if let Some(&first) = p.points.first() {
        b.move_to(first)?;
        for &q in &p.points[1..] {
            b.line_to(q)?;
        }
    }
    b.finish()
}
pub fn polygon_to_path(p: &Polygon) -> CoreResult<Path> {
    let mut b = PathBuilder::new();
    if let Some(&first) = p.points.first() {
        b.move_to(first)?;
        for &q in &p.points[1..] {
            b.line_to(q)?;
        }
        b.close()?;
    }
    b.finish()
}
