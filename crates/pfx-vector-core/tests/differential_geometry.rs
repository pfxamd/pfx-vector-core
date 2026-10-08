use pfx_vector_core::*;

fn assert_close(actual: f64, expected: f64, epsilon: f64) {
    assert!(
        (actual - expected).abs() <= epsilon,
        "actual={actual}, expected={expected}, epsilon={epsilon}"
    );
}

fn assert_vector_close(actual: Vector2, expected: Vector2, epsilon: f64) {
    assert_close(actual.x, expected.x, epsilon);
    assert_close(actual.y, expected.y, epsilon);
}

#[test]
fn line_frame_has_constant_tangent_normal_and_zero_curvature() {
    let segment = Segment::Line(LineSegment::new(
        Point2::new(0.0, 0.0),
        Point2::new(3.0, 4.0),
    ));

    let frame = segment_frame_at_t(segment, 0.5, Tolerance::default()).unwrap();

    assert!(frame.point.distance_to(Point2::new(1.5, 2.0)) <= 1.0e-12);
    assert_vector_close(frame.tangent, Vector2::new(0.6, 0.8), 1.0e-12);
    assert_vector_close(frame.normal, Vector2::new(-0.8, 0.6), 1.0e-12);
    assert_close(frame.curvature, 0.0, 1.0e-12);
    assert_close(frame.speed, 5.0, 1.0e-12);
}

#[test]
fn quadratic_frame_uses_analytic_second_derivative() {
    let curve = QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(2.0, 0.0),
    );

    let frame = segment_frame_at_t(Segment::Quadratic(curve), 0.5, Tolerance::default()).unwrap();

    assert!(frame.point.distance_to(Point2::new(1.0, 0.5)) <= 1.0e-12);
    assert_vector_close(frame.tangent, Vector2::new(1.0, 0.0), 1.0e-12);
    assert_vector_close(frame.normal, Vector2::new(0.0, 1.0), 1.0e-12);
    assert_close(frame.speed, 2.0, 1.0e-12);
    assert_close(frame.curvature, -1.0, 1.0e-12);
    assert_vector_close(
        curve.second_derivative_at(0.2),
        Vector2::new(0.0, -4.0),
        1.0e-12,
    );
}

#[test]
fn circular_arc_frame_has_expected_signed_curvature() {
    let arc = EllipticalArc::new(
        Point2::new(0.0, 0.0),
        10.0,
        10.0,
        Angle::degrees(0.0),
        Angle::degrees(0.0),
        Angle::degrees(180.0),
    );

    let frame = segment_frame_at_t(Segment::Arc(arc), 0.5, Tolerance::default()).unwrap();

    assert!(frame.point.distance_to(Point2::new(0.0, 10.0)) <= 1.0e-10);
    assert_close(frame.curvature, 0.1, 1.0e-12);
    assert_close(frame.speed, 10.0 * core::f64::consts::PI, 1.0e-10);
}

#[test]
fn reversal_flips_orientation_but_preserves_geometry_and_speed() {
    let segment = Segment::Cubic(CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(2.0, 4.0),
        Point2::new(5.0, -1.0),
        Point2::new(8.0, 2.0),
    ));
    let tolerance = Tolerance::default();

    let forward = segment_frame_at_t(segment, 0.3, tolerance).unwrap();
    let reverse = segment_frame_at_t(segment.reversed(), 0.7, tolerance).unwrap();

    assert!(forward.point.distance_to(reverse.point) <= 1.0e-10);
    assert_vector_close(reverse.tangent, -forward.tangent, 1.0e-10);
    assert_vector_close(reverse.normal, -forward.normal, 1.0e-10);
    assert_close(reverse.curvature, -forward.curvature, 1.0e-10);
    assert_close(reverse.speed, forward.speed, 1.0e-10);
}

#[test]
fn cubic_inflection_is_detected_at_native_parameter() {
    let segment = Segment::Cubic(CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(2.0, -1.0),
        Point2::new(3.0, 0.0),
    ));

    let inflections = segment_inflection_parameters(segment, Tolerance::default()).unwrap();

    assert_eq!(inflections.len(), 1);
    assert_close(inflections[0], 0.5, 1.0e-12);
}

#[test]
fn degenerate_tangent_has_defined_failure() {
    let segment = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 0.0),
        Point2::new(0.0, 0.0),
    ));

    assert_eq!(
        segment_frame_at_t(segment, 0.5, Tolerance::default()),
        Err(CoreError::DegenerateOperation)
    );
}

#[test]
fn path_frame_reports_original_location_and_endpoints() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(3.0, 4.0))
        .unwrap();
    let path = builder.finish().unwrap();

    let start = path_frame_at_length(&path, 0.0, Tolerance::default()).unwrap();
    let end = path_frame_at_length(&path, 5.0, Tolerance::default()).unwrap();

    assert_eq!(start.location.segment_index, 0);
    assert_close(start.location.t, 0.0, 1.0e-12);
    assert_close(end.location.t, 1.0, 1.0e-12);
    assert!(end.point.distance_to(Point2::new(3.0, 4.0)) <= 1.0e-12);
    assert_vector_close(
        tangent_at_length(&path, 2.0, Tolerance::default()).unwrap(),
        Vector2::new(0.6, 0.8),
        1.0e-12,
    );
}

#[test]
fn path_inflections_include_path_distance_coordinate() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(2.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(3.0, 1.0),
            Point2::new(4.0, -1.0),
            Point2::new(5.0, 0.0),
        )
        .unwrap();
    let path = builder.finish().unwrap();

    let inflections = path_inflections(&path, Tolerance::default()).unwrap();

    assert_eq!(inflections.len(), 1);
    assert_eq!(inflections[0].location.segment_index, 1);
    assert_close(inflections[0].location.t, 0.5, 1.0e-12);
    assert!(inflections[0].location.distance > 2.0);
}

#[test]
fn differential_geometry_is_translation_stable_at_large_coordinates() {
    let offset = 1.0e12;
    let segment = Segment::Cubic(CubicBezier::new(
        Point2::new(offset, offset),
        Point2::new(offset + 1.0, offset + 1.0),
        Point2::new(offset + 2.0, offset - 1.0),
        Point2::new(offset + 3.0, offset),
    ));

    let frame = segment_frame_at_t(segment, 0.5, Tolerance::default()).unwrap();
    let inflections = segment_inflection_parameters(segment, Tolerance::default()).unwrap();

    assert!(frame.curvature.is_finite());
    assert!(frame.speed.is_finite());
    assert_eq!(inflections.len(), 1);
    assert_close(inflections[0], 0.5, 1.0e-12);
}

#[test]
fn differential_api_rejects_invalid_parameter() {
    let segment = Segment::Line(LineSegment::new(
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 0.0),
    ));

    assert_eq!(
        segment_frame_at_t(segment, f64::NAN, Tolerance::default()),
        Err(CoreError::InvalidNumber)
    );
    assert_eq!(
        segment_frame_at_t(segment, 1.5, Tolerance::default()),
        Err(CoreError::InvalidGeometry)
    );
}
