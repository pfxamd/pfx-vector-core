use pfx_vector_core::*;

fn tolerance() -> Tolerance {
    Tolerance::default()
}

fn overlaps(result: &IntersectionResult) -> Vec<OverlapIntersection> {
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
fn line_and_collinear_quadratic_report_partial_overlap() {
    let line = Segment::Line(LineSegment::new(
        Point2::new(2.0, 0.0),
        Point2::new(8.0, 0.0),
    ));
    let curve = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(5.0, 0.0),
        Point2::new(10.0, 0.0),
    ));

    let result = intersect_segments(line, curve, tolerance()).unwrap();
    let overlaps = overlaps(&result);

    assert_eq!(overlaps.len(), 1, "{result:#?}");
    assert!((overlaps[0].range_a.min - 0.0).abs() < 1.0e-9);
    assert!((overlaps[0].range_a.max - 1.0).abs() < 1.0e-9);
    assert!((overlaps[0].range_b.min - 0.2).abs() < 1.0e-9);
    assert!((overlaps[0].range_b.max - 0.8).abs() < 1.0e-9);
}

#[test]
fn swapping_line_and_collinear_curve_swaps_overlap_ranges() {
    let line = Segment::Line(LineSegment::new(
        Point2::new(2.0, 0.0),
        Point2::new(8.0, 0.0),
    ));
    let curve = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(5.0, 0.0),
        Point2::new(10.0, 0.0),
    ));

    let forward = overlaps(&intersect_segments(line, curve, tolerance()).unwrap());
    let reverse = overlaps(&intersect_segments(curve, line, tolerance()).unwrap());

    assert_eq!(forward.len(), 1);
    assert_eq!(reverse.len(), 1);
    assert_eq!(forward[0].range_a, reverse[0].range_b);
    assert_eq!(forward[0].range_b, reverse[0].range_a);
}

#[test]
fn retraced_collinear_cubic_reports_multiple_overlap_intervals() {
    let line = Segment::Line(LineSegment::new(
        Point2::new(0.0, 0.0),
        Point2::new(10.0, 0.0),
    ));
    let curve = Segment::Cubic(CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(18.0, 0.0),
        Point2::new(-8.0, 0.0),
        Point2::new(10.0, 0.0),
    ));

    let result = intersect_segments(line, curve, tolerance()).unwrap();
    let overlaps = overlaps(&result);

    assert!(overlaps.len() >= 3, "{result:#?}");
    assert!(overlaps.iter().all(|overlap| {
        overlap.range_a.min >= 0.0
            && overlap.range_a.max <= 1.0
            && overlap.range_b.min >= 0.0
            && overlap.range_b.max <= 1.0
            && overlap.range_a.length() > 0.0
            && overlap.range_b.length() > 0.0
    }));
}

#[test]
fn nearly_collinear_quadratic_is_not_promoted_to_overlap() {
    let line = Segment::Line(LineSegment::new(
        Point2::new(0.0, 0.0),
        Point2::new(10.0, 0.0),
    ));
    let curve = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(5.0, 0.01),
        Point2::new(10.0, 0.0),
    ));

    let result = intersect_segments(line, curve, tolerance()).unwrap();

    assert!(
        result
            .intersections
            .iter()
            .all(|intersection| !matches!(intersection, Intersection::Overlap(_)))
    );
}
