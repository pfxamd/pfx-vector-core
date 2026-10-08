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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn randomized_cubic_measure_table_is_monotone(
        height in 2.0f64..80.0,
        width in 10.0f64..100.0,
        shape in -0.75f64..0.75,
    ) {
        let segment = Segment::Cubic(CubicBezier::new(
            Point2::new(0.0, 0.0),
            Point2::new(width * 0.25, height),
            Point2::new(width * 0.75, height * shape),
            Point2::new(width, 0.0),
        ));
        let table = SegmentMeasureTable::build(segment, Tolerance::default()).unwrap();
        let total = table.total_length();
        prop_assert!(total.is_finite() && total > 0.0);
        let mut previous = 0.0;
        for i in 0..=32 {
            let t = i as f64 / 32.0;
            let distance = table.length_at_parameter(t).unwrap();
            prop_assert!(distance.is_finite());
            prop_assert!(distance + 1.0e-8 >= previous);
            prop_assert!(distance <= total + 1.0e-8);
            previous = distance;
        }
    }

    #[test]
    fn randomized_elliptical_arc_measure_is_monotone(
        rx in 10.0f64..100.0,
        ry in 10.0f64..100.0,
        rotation in -150.0f64..150.0,
        sweep in 15.0f64..300.0,
    ) {
        let segment = Segment::Arc(EllipticalArc::new(
            Point2::new(0.0, 0.0),
            rx,
            ry,
            Angle::degrees(rotation),
            Angle::degrees(5.0),
            Angle::degrees(sweep),
        ));
        let table = SegmentMeasureTable::build(segment, Tolerance::default()).unwrap();
        let total = table.total_length();
        prop_assert!(total.is_finite() && total > 0.0);
        let mut previous = 0.0;
        for i in 0..=24 {
            let distance = table.length_at_parameter(i as f64 / 24.0).unwrap();
            prop_assert!(distance.is_finite());
            prop_assert!(distance + 1.0e-8 >= previous);
            prop_assert!(distance <= total + 1.0e-8);
            previous = distance;
        }
    }
}
