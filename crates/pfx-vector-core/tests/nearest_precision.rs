use pfx_vector_core::*;

fn assert_close(actual: f64, expected: f64, epsilon: f64) {
    assert!(
        (actual - expected).abs() <= epsilon,
        "actual={actual}, expected={expected}, epsilon={epsilon}"
    );
}

#[test]
fn quadratic_nearest_is_native_and_independent_of_flatness() {
    let curve = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(5.0, 10.0),
        Point2::new(10.0, 0.0),
    ));
    let tolerance = Tolerance {
        flatness: 100.0,
        ..Tolerance::default()
    };

    let result = closest_point_on_segment(curve, Point2::new(5.0, 8.0), tolerance).unwrap();

    assert_close(result.t, 0.5, 1.0e-10);
    assert!(result.point.distance_to(Point2::new(5.0, 5.0)) <= 1.0e-9);
    assert_close(result.distance, 3.0, 1.0e-9);
}

#[test]
fn cubic_nearest_solves_stationary_quintic() {
    let curve = Segment::Cubic(CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(0.0, 10.0),
        Point2::new(10.0, 10.0),
        Point2::new(10.0, 0.0),
    ));

    let result =
        closest_point_on_segment(curve, Point2::new(5.0, 12.0), Tolerance::default()).unwrap();

    assert_close(result.t, 0.5, 1.0e-9);
    assert!(result.point.distance_to(Point2::new(5.0, 7.5)) <= 1.0e-8);
    assert_close(result.distance, 4.5, 1.0e-8);
}

#[test]
fn elliptical_arc_nearest_uses_native_ellipse_geometry() {
    let arc = Segment::Arc(EllipticalArc::new(
        Point2::new(0.0, 0.0),
        10.0,
        5.0,
        Angle::degrees(0.0),
        Angle::degrees(0.0),
        Angle::degrees(180.0),
    ));

    let result =
        closest_point_on_segment(arc, Point2::new(0.0, 8.0), Tolerance::default()).unwrap();

    assert_close(result.t, 0.5, 1.0e-9);
    assert!(result.point.distance_to(Point2::new(0.0, 5.0)) <= 1.0e-8);
    assert_close(result.distance, 3.0, 1.0e-8);
}

#[test]
fn path_nearest_reports_original_segment_and_exact_distance_coordinate() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap()
        .quad_to(Point2::new(15.0, 10.0), Point2::new(20.0, 0.0))
        .unwrap();
    let path = builder.finish().unwrap();
    let tolerance = Tolerance {
        flatness: 100.0,
        ..Tolerance::default()
    };

    let result = closest_point(&path, Point2::new(15.0, 8.0), tolerance).unwrap();

    assert_eq!(result.location.subpath_index, 0);
    assert_eq!(result.location.segment_index, 1);
    assert_close(result.location.t, 0.5, 1.0e-9);
    assert!(result.location.distance > 10.0);
    assert!(result.location.distance < path_length(&path, tolerance).unwrap());
}

#[test]
fn nearest_is_stable_at_large_coordinates() {
    let offset = 1.0e12;
    let curve = Segment::Cubic(CubicBezier::new(
        Point2::new(offset, offset),
        Point2::new(offset + 20.0, offset + 40.0),
        Point2::new(offset + 40.0, offset + 40.0),
        Point2::new(offset + 60.0, offset),
    ));
    let point = Point2::new(offset + 30.0, offset + 50.0);

    let result = closest_point_on_segment(curve, point, Tolerance::default()).unwrap();

    assert!(result.point.is_finite());
    assert!(result.distance.is_finite());
    assert!(result.t >= 0.0 && result.t <= 1.0);
}

#[test]
fn nearest_rejects_non_finite_query() {
    let segment = Segment::Line(LineSegment::new(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 0.0),
    ));

    assert_eq!(
        closest_point_on_segment(segment, Point2::new(f64::NAN, 0.0), Tolerance::default()),
        Err(CoreError::InvalidNumber)
    );
}
