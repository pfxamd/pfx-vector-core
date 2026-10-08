//! Bounded deterministic load and numerical regression checks.
//! These tests are intentionally suitable for ordinary CI, not performance benchmarks.
use pfx_vector_core::*;

#[test]
fn large_polyline_measurement_has_stable_source_addresses() {
    const COUNT: usize = 4096;
    let mut builder = PathBuilder::new();
    builder.move_to(Point2::new(0.0, 0.0)).unwrap();
    for i in 1..=COUNT {
        builder.line_to(Point2::new(i as f64 * 3.0, 0.0)).unwrap();
    }
    let path = builder.finish().unwrap();
    let index = PathMeasureIndex::build(&path, Tolerance::default()).unwrap();
    assert_eq!(index.segment_count(), COUNT);
    assert_eq!(index.sample_count(), COUNT * 2);
    assert_eq!(index.total_length(), COUNT as f64 * 3.0);

    for i in 0..=COUNT {
        let distance = i as f64 * 3.0;
        let (point, location) = index.point_at_length(distance).unwrap();
        assert!((point.x - distance).abs() <= 1.0e-9);
        assert!(point.y.abs() <= 1.0e-9);
        assert_eq!(location.subpath_index, 0);
        assert!(location.segment_index < COUNT);
        assert!((location.distance - distance).abs() <= 1.0e-9);
    }
}

#[test]
fn many_curves_index_stays_finite_and_monotone() {
    const COUNT: usize = 128;
    let mut builder = PathBuilder::new();
    builder.move_to(Point2::new(0.0, 0.0)).unwrap();
    for i in 0..COUNT {
        let x = i as f64 * 10.0;
        builder
            .cubic_to(
                Point2::new(x + 2.0, 5.0),
                Point2::new(x + 8.0, -5.0),
                Point2::new(x + 10.0, 0.0),
            )
            .unwrap();
    }
    let path = builder.finish().unwrap();
    let index = PathMeasureIndex::build(&path, Tolerance::default()).unwrap();
    assert_eq!(index.segment_count(), COUNT);
    let total = index.total_length();
    assert!(total.is_finite() && total > 0.0);

    let mut previous_x = 0.0;
    for i in 0..=256 {
        let distance = total * i as f64 / 256.0;
        let (point, location) = index.point_at_length(distance).unwrap();
        assert!(point.x.is_finite() && point.y.is_finite());
        assert!(location.t.is_finite() && (0.0..=1.0).contains(&location.t));
        assert!(point.x + 1.0e-7 >= previous_x);
        previous_x = point.x;
        let reverse = index
            .distance_at_location(location.subpath_index, location.segment_index, location.t)
            .unwrap();
        assert!((reverse - distance).abs() <= 1.0e-5);
    }
}

#[test]
fn repeated_index_queries_are_deterministic() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(20.0, 60.0),
            Point2::new(80.0, -50.0),
            Point2::new(100.0, 0.0),
        )
        .unwrap();
    let path = builder.finish().unwrap();
    let index = PathMeasureIndex::build(&path, Tolerance::default()).unwrap();
    let requested = index.total_length() * 0.431;
    let first = index.point_at_length(requested).unwrap();
    for _ in 0..1024 {
        assert_eq!(index.point_at_length(requested).unwrap(), first);
    }
}
