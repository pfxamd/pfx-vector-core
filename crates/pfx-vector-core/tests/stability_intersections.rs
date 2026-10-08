//! Deterministic intersection invariants under translation and operand reversal.
use pfx_vector_core::*;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn crossing_lines_have_symmetric_point_hits(
        offset_x in -1.0e5f64..1.0e5,
        offset_y in -1.0e5f64..1.0e5,
        scale in 1.0f64..100.0,
    ) {
        let point = |x: f64, y: f64| Point2::new(offset_x + x * scale, offset_y + y * scale);
        let a = Segment::Line(LineSegment::new(point(-2.0, 0.0), point(2.0, 0.0)));
        let b = Segment::Line(LineSegment::new(point(0.0, -2.0), point(0.0, 2.0)));
        let tolerance = Tolerance::default();
        let ab = intersect_segments(a, b, tolerance).unwrap();
        let ba = intersect_segments(b, a, tolerance).unwrap();
        prop_assert!(!ab.intersections.is_empty());
        prop_assert_eq!(ab.intersections.len(), ba.intersections.len());
    }
}
