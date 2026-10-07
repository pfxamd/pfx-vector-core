use pfx_vector_core::*;

fn point_hits(result: &IntersectionResult) -> Vec<PointIntersection> {
    result
        .intersections
        .iter()
        .filter_map(|intersection| match intersection {
            Intersection::Point(point) => Some(*point),
            Intersection::Overlap(_) => None,
        })
        .collect()
}

fn default_tolerance() -> Tolerance {
    Tolerance {
        flatness: 1.0e-5,
        ..Tolerance::default()
    }
}

#[test]
fn quadratic_quadratic_finds_two_crossings() {
    let arch = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(50.0, 100.0),
        Point2::new(100.0, 0.0),
    ));
    let horizontal = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(0.0, 25.0),
        Point2::new(50.0, 25.0),
        Point2::new(100.0, 25.0),
    ));

    let result = intersect_segments(arch, horizontal, default_tolerance()).unwrap();
    let hits = point_hits(&result);

    assert_eq!(hits.len(), 2);
    assert!(
        hits.iter()
            .all(|hit| hit.kind == IntersectionKind::Crossing)
    );
    assert!(hits.iter().all(|hit| (hit.point.y - 25.0).abs() < 1.0e-4));
    assert!(hits[0].parameter_a < hits[1].parameter_a);
}

#[test]
fn quadratic_cubic_finds_curve_pair_crossings() {
    let quadratic = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(50.0, 100.0),
        Point2::new(100.0, 0.0),
    ));
    let cubic = Segment::Cubic(CubicBezier::new(
        Point2::new(0.0, 25.0),
        Point2::new(33.0, 25.0),
        Point2::new(66.0, 25.0),
        Point2::new(100.0, 25.0),
    ));

    let result = intersect_segments(quadratic, cubic, default_tolerance()).unwrap();
    let hits = point_hits(&result);

    assert_eq!(hits.len(), 2);
    assert!(hits.iter().all(|hit| (hit.point.y - 25.0).abs() < 1.0e-4));
}

#[test]
fn cubic_cubic_finds_single_crossing() {
    let horizontal = Segment::Cubic(CubicBezier::new(
        Point2::new(-10.0, 0.0),
        Point2::new(-3.0, 0.0),
        Point2::new(3.0, 0.0),
        Point2::new(10.0, 0.0),
    ));
    let vertical = Segment::Cubic(CubicBezier::new(
        Point2::new(0.0, -10.0),
        Point2::new(0.0, -3.0),
        Point2::new(0.0, 3.0),
        Point2::new(0.0, 10.0),
    ));

    let result = intersect_segments(horizontal, vertical, default_tolerance()).unwrap();
    let hits = point_hits(&result);

    assert_eq!(hits.len(), 1);
    assert!(hits[0].point.distance_to(Point2::new(0.0, 0.0)) < 1.0e-5);
    assert_eq!(hits[0].kind, IntersectionKind::Crossing);
}

#[test]
fn bezier_arc_finds_both_semicircle_endpoints() {
    let quadratic = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(-20.0, 0.0),
        Point2::new(0.0, 0.0),
        Point2::new(20.0, 0.0),
    ));
    let arc = Segment::Arc(EllipticalArc::new(
        Point2::new(0.0, 0.0),
        10.0,
        10.0,
        Angle::radians(0.0),
        Angle::radians(0.0),
        Angle::radians(core::f64::consts::PI),
    ));

    let result = intersect_segments(quadratic, arc, default_tolerance()).unwrap();
    let hits = point_hits(&result);

    assert_eq!(hits.len(), 2);
    assert!(
        hits.iter()
            .any(|hit| hit.point.distance_to(Point2::new(10.0, 0.0)) < 1.0e-4)
    );
    assert!(
        hits.iter()
            .any(|hit| hit.point.distance_to(Point2::new(-10.0, 0.0)) < 1.0e-4)
    );
}

#[test]
fn arc_arc_finds_two_circle_crossings() {
    let left = Segment::Arc(EllipticalArc::new(
        Point2::new(0.0, 0.0),
        10.0,
        10.0,
        Angle::radians(0.0),
        Angle::radians(0.0),
        Angle::radians(core::f64::consts::TAU),
    ));
    let right = Segment::Arc(EllipticalArc::new(
        Point2::new(10.0, 0.0),
        10.0,
        10.0,
        Angle::radians(0.0),
        Angle::radians(0.0),
        Angle::radians(core::f64::consts::TAU),
    ));

    let result = intersect_segments(left, right, default_tolerance()).unwrap();
    let hits = point_hits(&result);

    assert_eq!(hits.len(), 2);
    assert!(hits.iter().all(|hit| (hit.point.x - 5.0).abs() < 1.0e-4));
    assert!(hits.iter().any(|hit| hit.point.y > 8.0));
    assert!(hits.iter().any(|hit| hit.point.y < -8.0));
}

#[test]
fn interior_arc_tangency_is_classified_as_tangent() {
    let left = Segment::Arc(EllipticalArc::new(
        Point2::new(0.0, 0.0),
        10.0,
        10.0,
        Angle::radians(0.0),
        Angle::radians(-core::f64::consts::FRAC_PI_2),
        Angle::radians(core::f64::consts::PI),
    ));
    let right = Segment::Arc(EllipticalArc::new(
        Point2::new(20.0, 0.0),
        10.0,
        10.0,
        Angle::radians(0.0),
        Angle::radians(core::f64::consts::FRAC_PI_2),
        Angle::radians(core::f64::consts::PI),
    ));

    let result = intersect_segments(left, right, default_tolerance()).unwrap();
    let hits = point_hits(&result);

    assert_eq!(hits.len(), 1);
    assert!(hits[0].point.distance_to(Point2::new(10.0, 0.0)) < 1.0e-4);
    assert_eq!(hits[0].kind, IntersectionKind::Tangent);
}

#[test]
fn line_quadratic_tangency_is_classified_as_tangent() {
    let line = Segment::Line(LineSegment::new(
        Point2::new(0.0, 50.0),
        Point2::new(100.0, 50.0),
    ));
    let quadratic = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(50.0, 100.0),
        Point2::new(100.0, 0.0),
    ));

    let result = intersect_segments(line, quadratic, default_tolerance()).unwrap();
    let hits = point_hits(&result);

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].kind, IntersectionKind::Tangent);
    assert!(hits[0].point.distance_to(Point2::new(50.0, 50.0)) < 1.0e-8);
}

#[test]
fn reversed_cubic_reports_overlap() {
    let cubic = CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(25.0, 50.0),
        Point2::new(75.0, 50.0),
        Point2::new(100.0, 0.0),
    );

    let result = intersect_segments(
        Segment::Cubic(cubic),
        Segment::Cubic(cubic.reversed()),
        default_tolerance(),
    )
    .unwrap();

    let Some(Intersection::Overlap(overlap)) = result.intersections.first() else {
        panic!("expected overlap");
    };

    assert_eq!(overlap.range_a, Interval::new(0.0, 1.0).unwrap());
    assert_eq!(overlap.range_b, Interval::new(0.0, 1.0).unwrap());
}

#[test]
fn partially_overlapping_arcs_report_parameter_ranges() {
    let first = Segment::Arc(EllipticalArc::new(
        Point2::new(0.0, 0.0),
        10.0,
        10.0,
        Angle::radians(0.0),
        Angle::radians(0.0),
        Angle::radians(core::f64::consts::PI),
    ));
    let second = Segment::Arc(EllipticalArc::new(
        Point2::new(0.0, 0.0),
        10.0,
        10.0,
        Angle::radians(0.0),
        Angle::radians(core::f64::consts::FRAC_PI_2),
        Angle::radians(core::f64::consts::PI),
    ));

    let result = intersect_segments(first, second, default_tolerance()).unwrap();

    let Some(Intersection::Overlap(overlap)) = result.intersections.first() else {
        panic!("expected overlap");
    };

    assert!((overlap.range_a.min - 0.5).abs() < 1.0e-10);
    assert!((overlap.range_a.max - 1.0).abs() < 1.0e-10);
    assert!((overlap.range_b.min - 0.0).abs() < 1.0e-10);
    assert!((overlap.range_b.max - 0.5).abs() < 1.0e-10);
}
