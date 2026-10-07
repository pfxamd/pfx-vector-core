use pfx_vector_core::*;

#[test]
fn retraced_collinear_cubic_length_uses_total_variation() {
    let curve = Segment::Cubic(CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(18.0, 0.0),
        Point2::new(-8.0, 0.0),
        Point2::new(10.0, 0.0),
    ));

    let length = segment_length(curve, Tolerance::default()).unwrap();

    assert!(length.is_finite());
    assert!(length > 10.0);
}

#[test]
fn spatial_index_builds_for_retraced_collinear_cubic() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(18.0, 0.0),
            Point2::new(-8.0, 0.0),
            Point2::new(10.0, 0.0),
        )
        .unwrap()
        .line_to(Point2::new(10.0, -10.0))
        .unwrap()
        .line_to(Point2::new(0.0, -10.0))
        .unwrap()
        .close()
        .unwrap();
    let path = builder.finish().unwrap();

    let index = PathSpatialIndex::build(&path, Tolerance::default()).unwrap();

    assert!(index.edge_count() > 0);
}
