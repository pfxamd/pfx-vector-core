use pfx_vector_core::*;

#[test]
fn tiny_and_large_valid_line_segments_remain_finite() {
    for (start_x, delta) in [(0.0, 1.0e-9), (1.0e12, 1.0), (-1.0e12, 1024.0)] {
        let mut builder = PathBuilder::new();
        builder.move_to(Point2::new(start_x, 0.0)).unwrap();
        builder.line_to(Point2::new(start_x + delta, 0.0)).unwrap();
        let path = builder.finish().unwrap();
        let index = PathMeasureIndex::build(&path, Tolerance::default()).unwrap();
        assert!(index.total_length().is_finite());
        for fraction in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let (point, location) = index.point_at_length(index.total_length() * fraction).unwrap();
            assert!(point.x.is_finite() && point.y.is_finite());
            assert!(location.t.is_finite());
        }
    }
}

#[test]
fn zero_length_line_is_bounded_and_deterministic() {
    let mut builder = PathBuilder::new();
    builder.move_to(Point2::new(4.0, -2.0)).unwrap();
    builder.line_to(Point2::new(4.0, -2.0)).unwrap();
    let path = builder.finish().unwrap();
    let index = PathMeasureIndex::build(&path, Tolerance::default()).unwrap();
    assert_eq!(index.total_length(), 0.0);
    let first = index.point_at_length(0.0).unwrap();
    for _ in 0..32 {
        assert_eq!(index.point_at_length(0.0).unwrap(), first);
    }
}
