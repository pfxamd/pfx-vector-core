use pfx_vector_core::*;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn randomized_line_measurement_remains_consistent(
        x in -1.0e5f64..1.0e5,
        y in -1.0e5f64..1.0e5,
        dx in -1.0e3f64..1.0e3,
        dy in -1.0e3f64..1.0e3,
        t in 0.0f64..1.0,
    ) {
        let start = Point2::new(x, y);
        let end = Point2::new(x + dx, y + dy);
        let segment = Segment::Line(LineSegment::new(start, end));
        let table = SegmentMeasureTable::build(segment, Tolerance::default()).unwrap();
        let total = table.total_length();

        prop_assert!(total.is_finite() && total >= 0.0);
        prop_assert_eq!(table.sample_count(), 2);
        let distance = table.length_at_parameter(t).unwrap();
        prop_assert!(distance.is_finite() && distance >= 0.0 && distance <= total);
        if total > 0.0 {
            let recovered = table.parameter_at_length(distance).unwrap();
            prop_assert!((recovered - t).abs() <= 1.0e-12);
        }
        let point = table.point_at_length(distance).unwrap();
        let expected = segment.point_at(t);
        prop_assert!(point.distance_to(expected) <= 1.0e-7);
    }
}
