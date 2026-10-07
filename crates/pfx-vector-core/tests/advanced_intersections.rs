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

    assert_eq!(hits.len(), 1, "tangent hits: {hits:#?}");
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

#[test]
fn non_dyadic_quadratic_tangency_is_stable() {
    let tangent_parameter = 0.3;
    let y0 = tangent_parameter * tangent_parameter;
    let y1 = y0 - tangent_parameter;
    let y2 = (1.0 - tangent_parameter) * (1.0 - tangent_parameter);

    let parabola = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(0.0, y0),
        Point2::new(0.5, y1),
        Point2::new(1.0, y2),
    ));
    let axis = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(0.5, 0.0),
        Point2::new(1.0, 0.0),
    ));

    let result = intersect_segments(parabola, axis, default_tolerance()).unwrap();
    let hits = point_hits(&result);

    assert_eq!(hits.len(), 1, "non-dyadic tangent hits: {hits:#?}");
    assert_eq!(hits[0].kind, IntersectionKind::Tangent);
    assert!((hits[0].parameter_a - tangent_parameter).abs() < 1.0e-6);
    assert!(
        hits[0]
            .point
            .distance_to(Point2::new(tangent_parameter, 0.0))
            < 1.0e-7
    );
}

#[test]
fn near_tangent_curves_do_not_create_false_intersection() {
    let tangent_parameter = 0.3;
    let gap = 5.0e-9;
    let y0 = tangent_parameter * tangent_parameter + gap;
    let y1 = tangent_parameter * tangent_parameter - tangent_parameter + gap;
    let y2 = (1.0 - tangent_parameter) * (1.0 - tangent_parameter) + gap;

    let parabola = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(0.0, y0),
        Point2::new(0.5, y1),
        Point2::new(1.0, y2),
    ));
    let axis = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(0.5, 0.0),
        Point2::new(1.0, 0.0),
    ));

    let result = intersect_segments(parabola, axis, default_tolerance()).unwrap();

    assert!(result.intersections.is_empty(), "{result:#?}");
}

#[test]
fn advanced_intersections_are_symmetric() {
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

    let forward = point_hits(&intersect_segments(quadratic, cubic, default_tolerance()).unwrap());
    let reverse = point_hits(&intersect_segments(cubic, quadratic, default_tolerance()).unwrap());

    assert_eq!(forward.len(), reverse.len());

    for (left, right) in forward.iter().zip(reverse.iter()) {
        assert!(left.point.distance_to(right.point) < 1.0e-7);
        assert!((left.parameter_a - right.parameter_b).abs() < 1.0e-7);
        assert!((left.parameter_b - right.parameter_a).abs() < 1.0e-7);
        assert_eq!(left.kind, right.kind);
    }
}

#[test]
fn advanced_intersections_survive_large_translation() {
    let offset = 1.0e9;
    let horizontal = Segment::Cubic(CubicBezier::new(
        Point2::new(offset - 10.0, offset),
        Point2::new(offset - 3.0, offset),
        Point2::new(offset + 3.0, offset),
        Point2::new(offset + 10.0, offset),
    ));
    let vertical = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(offset, offset - 10.0),
        Point2::new(offset, offset),
        Point2::new(offset, offset + 10.0),
    ));

    let result = intersect_segments(horizontal, vertical, default_tolerance()).unwrap();
    let hits = point_hits(&result);

    assert_eq!(hits.len(), 1, "{hits:#?}");
    assert!(hits[0].point.distance_to(Point2::new(offset, offset)) < 1.0e-4);
}

fn cubic_subcurve(curve: CubicBezier, t0: f64, t1: f64) -> CubicBezier {
    let (_, right) = curve.split(t0);
    let local = (t1 - t0) / (1.0 - t0);
    right.split(local).0
}

fn elevate_quadratic_for_test(curve: QuadraticBezier) -> CubicBezier {
    CubicBezier::new(
        curve.p0,
        curve.p0.lerp(curve.p1, 2.0 / 3.0),
        curve.p1.lerp(curve.p2, 1.0 / 3.0),
        curve.p2,
    )
}

#[test]
fn quadratic_subcurve_reports_partial_overlap() {
    let parent = QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(50.0, 80.0),
        Point2::new(100.0, 0.0),
    );
    let child = parent.subcurve(0.25, 0.75);

    let result = intersect_segments(
        Segment::Quadratic(parent),
        Segment::Quadratic(child),
        default_tolerance(),
    )
    .unwrap();

    let Some(Intersection::Overlap(overlap)) = result.intersections.first() else {
        panic!("expected partial quadratic overlap: {result:#?}");
    };

    assert!((overlap.range_a.min - 0.25).abs() < 1.0e-8);
    assert!((overlap.range_a.max - 0.75).abs() < 1.0e-8);
    assert!(overlap.range_b.min.abs() < 1.0e-8);
    assert!((overlap.range_b.max - 1.0).abs() < 1.0e-8);
}

#[test]
fn cubic_middle_overlap_is_detected_when_neither_segment_contains_the_other() {
    let base = CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(20.0, 90.0),
        Point2::new(80.0, -70.0),
        Point2::new(120.0, 10.0),
    );
    let first = cubic_subcurve(base, 0.0, 0.7);
    let second = cubic_subcurve(base, 0.3, 1.0);

    let result = intersect_segments(
        Segment::Cubic(first),
        Segment::Cubic(second),
        default_tolerance(),
    )
    .unwrap();

    let Some(Intersection::Overlap(overlap)) = result.intersections.first() else {
        panic!("expected middle cubic overlap: {result:#?}");
    };

    assert!((overlap.range_a.min - (0.3 / 0.7)).abs() < 1.0e-7);
    assert!((overlap.range_a.max - 1.0).abs() < 1.0e-8);
    assert!(overlap.range_b.min.abs() < 1.0e-8);
    assert!((overlap.range_b.max - (0.4 / 0.7)).abs() < 1.0e-7);
}

#[test]
fn reversed_cubic_subcurve_reports_partial_overlap() {
    let parent = CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(30.0, 70.0),
        Point2::new(80.0, 60.0),
        Point2::new(120.0, 0.0),
    );
    let child = cubic_subcurve(parent, 0.2, 0.8).reversed();

    let result = intersect_segments(
        Segment::Cubic(parent),
        Segment::Cubic(child),
        default_tolerance(),
    )
    .unwrap();

    let Some(Intersection::Overlap(overlap)) = result.intersections.first() else {
        panic!("expected reversed partial cubic overlap: {result:#?}");
    };

    assert!((overlap.range_a.min - 0.2).abs() < 1.0e-7);
    assert!((overlap.range_a.max - 0.8).abs() < 1.0e-7);
    assert!(overlap.range_b.min.abs() < 1.0e-8);
    assert!((overlap.range_b.max - 1.0).abs() < 1.0e-8);
}

#[test]
fn degree_elevated_quadratic_and_cubic_report_overlap() {
    let quadratic = QuadraticBezier::new(
        Point2::new(-10.0, 4.0),
        Point2::new(35.0, 75.0),
        Point2::new(90.0, -5.0),
    );
    let elevated = elevate_quadratic_for_test(quadratic);
    let partial = cubic_subcurve(elevated, 0.15, 0.85);

    let result = intersect_segments(
        Segment::Quadratic(quadratic),
        Segment::Cubic(partial),
        default_tolerance(),
    )
    .unwrap();

    let Some(Intersection::Overlap(overlap)) = result.intersections.first() else {
        panic!("expected cross-degree overlap: {result:#?}");
    };

    assert!((overlap.range_a.min - 0.15).abs() < 1.0e-7);
    assert!((overlap.range_a.max - 0.85).abs() < 1.0e-7);
    assert!(overlap.range_b.min.abs() < 1.0e-8);
    assert!((overlap.range_b.max - 1.0).abs() < 1.0e-8);
}

fn overlap_hits(result: &IntersectionResult) -> Vec<OverlapIntersection> {
    result
        .intersections
        .iter()
        .filter_map(|intersection| match intersection {
            Intersection::Overlap(overlap) => Some(*overlap),
            Intersection::Point(_) => None,
        })
        .collect()
}

#[test]
fn full_circles_with_different_rotation_cover_the_full_parameter_domain() {
    let first = Segment::Arc(EllipticalArc::new(
        Point2::new(0.0, 0.0),
        12.0,
        12.0,
        Angle::degrees(0.0),
        Angle::degrees(0.0),
        Angle::degrees(360.0),
    ));
    let second = Segment::Arc(EllipticalArc::new(
        Point2::new(0.0, 0.0),
        12.0,
        12.0,
        Angle::degrees(60.0),
        Angle::degrees(0.0),
        Angle::degrees(360.0),
    ));

    let result = intersect_segments(first, second, default_tolerance()).unwrap();
    let overlaps = overlap_hits(&result);

    assert_eq!(overlaps.len(), 2, "{result:#?}");
    let coverage_a: f64 = overlaps
        .iter()
        .map(|overlap| overlap.range_a.length())
        .sum();
    let coverage_b: f64 = overlaps
        .iter()
        .map(|overlap| overlap.range_b.length())
        .sum();
    assert!((coverage_a - 1.0).abs() < 1.0e-9);
    assert!((coverage_b - 1.0).abs() < 1.0e-9);
}

#[test]
fn swapped_ellipse_axes_and_rotation_report_same_overlap() {
    let first = Segment::Arc(EllipticalArc::new(
        Point2::new(5.0, -3.0),
        20.0,
        10.0,
        Angle::degrees(0.0),
        Angle::degrees(0.0),
        Angle::degrees(180.0),
    ));
    let second = Segment::Arc(EllipticalArc::new(
        Point2::new(5.0, -3.0),
        10.0,
        20.0,
        Angle::degrees(-90.0),
        Angle::degrees(90.0),
        Angle::degrees(180.0),
    ));

    let result = intersect_segments(first, second, default_tolerance()).unwrap();
    let overlaps = overlap_hits(&result);

    assert_eq!(overlaps.len(), 1, "{result:#?}");
    assert!((overlaps[0].range_a.length() - 1.0).abs() < 1.0e-9);
    assert!((overlaps[0].range_b.length() - 1.0).abs() < 1.0e-9);
}

#[test]
fn multi_revolution_arc_reports_each_parameter_overlap() {
    let double_turn = Segment::Arc(EllipticalArc::new(
        Point2::new(0.0, 0.0),
        10.0,
        6.0,
        Angle::degrees(25.0),
        Angle::degrees(0.0),
        Angle::degrees(720.0),
    ));
    let single_turn = Segment::Arc(EllipticalArc::new(
        Point2::new(0.0, 0.0),
        10.0,
        6.0,
        Angle::degrees(25.0),
        Angle::degrees(0.0),
        Angle::degrees(360.0),
    ));

    let result = intersect_segments(double_turn, single_turn, default_tolerance()).unwrap();
    let overlaps = overlap_hits(&result);

    assert_eq!(overlaps.len(), 2, "{result:#?}");
    assert!((overlaps[0].range_a.min - 0.0).abs() < 1.0e-9);
    assert!((overlaps[0].range_a.max - 0.5).abs() < 1.0e-9);
    assert!((overlaps[1].range_a.min - 0.5).abs() < 1.0e-9);
    assert!((overlaps[1].range_a.max - 1.0).abs() < 1.0e-9);
    assert!(
        overlaps
            .iter()
            .all(|overlap| (overlap.range_b.length() - 1.0).abs() < 1.0e-9)
    );
}

#[test]
fn reversed_multi_revolution_arc_keeps_deterministic_ranges() {
    let forward = Segment::Arc(EllipticalArc::new(
        Point2::new(0.0, 0.0),
        14.0,
        7.0,
        Angle::degrees(15.0),
        Angle::degrees(-30.0),
        Angle::degrees(720.0),
    ));
    let reverse = Segment::Arc(EllipticalArc::new(
        Point2::new(0.0, 0.0),
        14.0,
        7.0,
        Angle::degrees(15.0),
        Angle::degrees(330.0),
        Angle::degrees(-360.0),
    ));

    let result = intersect_segments(forward, reverse, default_tolerance()).unwrap();
    let overlaps = overlap_hits(&result);

    assert_eq!(overlaps.len(), 2, "{result:#?}");
    assert!(overlaps[0].range_a.min <= overlaps[1].range_a.min);
    assert!(
        overlaps
            .iter()
            .all(|overlap| overlap.range_a.length() > 0.49)
    );
    assert!(
        overlaps
            .iter()
            .all(|overlap| (overlap.range_b.length() - 1.0).abs() < 1.0e-9)
    );
}

#[test]
fn partial_overlap_ranges_are_symmetric_when_inputs_are_swapped() {
    let base = CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(25.0, 80.0),
        Point2::new(90.0, -60.0),
        Point2::new(130.0, 15.0),
    );
    let first = Segment::Cubic(cubic_subcurve(base, 0.05, 0.72));
    let second = Segment::Cubic(cubic_subcurve(base, 0.31, 0.94));

    let forward = overlap_hits(&intersect_segments(first, second, default_tolerance()).unwrap());
    let reverse = overlap_hits(&intersect_segments(second, first, default_tolerance()).unwrap());

    assert_eq!(forward.len(), 1);
    assert_eq!(reverse.len(), 1);
    assert!((forward[0].range_a.min - reverse[0].range_b.min).abs() < 1.0e-8);
    assert!((forward[0].range_a.max - reverse[0].range_b.max).abs() < 1.0e-8);
    assert!((forward[0].range_b.min - reverse[0].range_a.min).abs() < 1.0e-8);
    assert!((forward[0].range_b.max - reverse[0].range_a.max).abs() < 1.0e-8);
}

#[test]
fn partial_cubic_overlap_survives_large_translation() {
    let offset = 1.0e9;
    let base = CubicBezier::new(
        Point2::new(offset, offset),
        Point2::new(offset + 30.0, offset + 90.0),
        Point2::new(offset + 100.0, offset - 70.0),
        Point2::new(offset + 150.0, offset + 20.0),
    );
    let child = cubic_subcurve(base, 0.2, 0.8);

    let result = intersect_segments(
        Segment::Cubic(base),
        Segment::Cubic(child),
        default_tolerance(),
    )
    .unwrap();
    let overlaps = overlap_hits(&result);

    assert_eq!(overlaps.len(), 1, "{result:#?}");
    assert!((overlaps[0].range_a.min - 0.2).abs() < 1.0e-6);
    assert!((overlaps[0].range_a.max - 0.8).abs() < 1.0e-6);
}
