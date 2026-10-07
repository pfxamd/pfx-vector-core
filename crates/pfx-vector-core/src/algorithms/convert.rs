use crate::{
    Angle, Circle, CoreResult, Ellipse, EllipticalArc, Path, PathBuilder, Point2, Polygon, Polyline,
    Rect, RoundedRect, Tolerance,
};
pub fn rect_to_path(rect: Rect) -> CoreResult<Path> {
    let mut builder = PathBuilder::new();
    builder.move_to(Point2::new(rect.x, rect.y))? .line_to(Point2::new(rect.x + rect.width, rect.y))? .line_to(Point2::new(rect.x + rect.width,
    rect.y + rect.height))? .line_to(Point2::new(rect.x, rect.y + rect.height))? .close()?;
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
    b.move_to(Point2::new(r.x + rx, r.y))? .line_to(Point2::new(r.x + r.width - rx, r.y))?;
    let centers = [(Point2::new(r.x + r.width - rx, r.y + ry), -core::f64::consts::FRAC_PI_2), (Point2::new(r.x + r.width - rx,
    r.y + r.height - ry), 0.0), (Point2::new(r.x + rx, r.y + r.height - ry), core::f64::consts::FRAC_PI_2),
    (Point2::new(r.x + rx, r.y + ry), core::f64::consts::PI),];
    for (index, (center, start)) in centers.into_iter().enumerate() {
        let arc = EllipticalArc::new(center, rx, ry, Angle::radians(0.0), Angle::radians(start), Angle::radians(core::f64::consts::FRAC_PI_2),
        )?;
        b.arc_to(arc)?;
        match index {
            0 => { b.line_to(Point2::new(r.x + r.width, r.y + r.height - ry))?; }
            1 => { b.line_to(Point2::new(r.x + rx, r.y + r.height))?; }
            2 => { b.line_to(Point2::new(r.x, r.y + ry))?; }
            _ => {}
        }
    }
    b.close()?;
    b.finish()
}
pub fn ellipse_to_path(ellipse: Ellipse) -> CoreResult<Path> {
    let mut b = PathBuilder::with_tolerance(Tolerance::default());
    let start = Point2::new(ellipse.center.x + ellipse.radius_x * ellipse.rotation.as_radians().cos(),
    ellipse.center.y + ellipse.radius_x * ellipse.rotation.as_radians().sin(),);
    b.move_to(start)?;
    b.arc_to(EllipticalArc::new(ellipse.center, ellipse.radius_x, ellipse.radius_y, ellipse.rotation,
    Angle::radians(0.0), Angle::radians(core::f64::consts::PI),)?)?;
    b.arc_to(EllipticalArc::new(ellipse.center, ellipse.radius_x, ellipse.radius_y, ellipse.rotation,
    Angle::radians(core::f64::consts::PI), Angle::radians(core::f64::consts::PI),)?)?;
    b.close()?;
    b.finish()
}
pub fn circle_to_path(circle: Circle) -> CoreResult<Path> {
    ellipse_to_path(Ellipse::new(circle.center, circle.radius, circle.radius, Angle::radians(0.0))?)
}
pub fn polyline_to_path(polyline: &Polyline) -> CoreResult<Path> {
    let mut b = PathBuilder::new();
    if let Some(&first) = polyline.points.first() {
        b.move_to(first)?;
        for &point in &polyline.points[1..] { b.line_to(point)?; }
    }
    b.finish()
}
pub fn polygon_to_path(polygon: &Polygon) -> CoreResult<Path> {
    let mut b = PathBuilder::new();
    if let Some(&first) = polygon.points.first() {
        b.move_to(first)?;
        for &point in &polygon.points[1..] { b.line_to(point)?; }
        b.close()?;
    }
    b.finish()
}
