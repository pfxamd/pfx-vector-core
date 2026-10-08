use pfx_vector_core::*;

fn assert_close(actual: f64, expected: f64, epsilon: f64) {
    assert!(
        (actual - expected).abs() <= epsilon,
        "actual={actual}, expected={expected}, epsilon={epsilon}"
    );
}

fn curved_path() -> Path {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(20.0, 80.0),
            Point2::new(80.0, -60.0),
            Point2::new(120.0, 20.0),
        )
        .unwrap()
        .quad_to(Point2::new(150.0, 60.0), Point2::new(180.0, 0.0))
        .unwrap();
    builder.finish().unwrap()
}

#[test]
fn line_measure_table_is_exact_and_minimal() {
    let segment = Segment::Line(LineSegment::new(
        Point2::new(0.0, 0.0),
        Point2::new(3.0, 4.0),
    ));
    let table = SegmentMeasureTable::build(segment, Tolerance::default()).unwrap();

    assert_eq!(table.sample_count(), 2);
    assert_close(table.total_length(), 5.0, 1.0e-12);
    assert_close(table.length_at_parameter(0.25).unwrap(), 1.25, 1.0e-12);
    assert_close(table.parameter_at_length(3.75).unwrap(), 0.75, 1.0e-12);
}

#[test]
fn cubic_measure_table_round_trips_parameter_and_length() {
    let segment = Segment::Cubic(CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(20.0, 80.0),
        Point2::new(80.0, -60.0),
        Point2::new(120.0, 20.0),
    ));
    let table = SegmentMeasureTable::build(segment, Tolerance::default()).unwrap();

    assert!(table.sample_count() > 2);
    for t in [0.0, 0.1, 0.37, 0.5, 0.83, 1.0] {
        let distance = table.length_at_parameter(t).unwrap();
        let recovered = table.parameter_at_length(distance).unwrap();
        assert_close(recovered, t, 2.0e-7);
    }
}

#[test]
fn elliptical_arc_measure_table_round_trips() {
    let arc = EllipticalArc::new(
        Point2::new(10.0, -20.0),
        80.0,
        25.0,
        Angle::degrees(27.0),
        Angle::degrees(-35.0),
        Angle::degrees(250.0),
    );
    let table = SegmentMeasureTable::build(Segment::Arc(arc), Tolerance::default()).unwrap();

    for t in [0.05, 0.25, 0.6, 0.95] {
        let distance = table.length_at_parameter(t).unwrap();
        let recovered = table.parameter_at_length(distance).unwrap();
        assert_close(recovered, t, 2.0e-7);
    }
}

#[test]
fn path_measure_index_matches_one_shot_queries() {
    let path = curved_path();
    let tolerance = Tolerance::default();
    let index = PathMeasureIndex::build(&path, tolerance).unwrap();
    let total = path_length(&path, tolerance).unwrap();

    assert_close(index.total_length(), total, 1.0e-8);
    for fraction in [0.0, 0.1, 0.33, 0.5, 0.8, 1.0] {
        let distance = total * fraction;
        let (expected_point, expected_location) =
            point_at_length(&path, distance, tolerance).unwrap();
        let (actual_point, actual_location) = index.point_at_length(distance).unwrap();

        assert!(expected_point.distance_to(actual_point) <= 2.0e-6);
        assert_eq!(
            actual_location.subpath_index,
            expected_location.subpath_index
        );
        assert_eq!(
            actual_location.segment_index,
            expected_location.segment_index
        );
        assert_close(actual_location.distance, expected_location.distance, 1.0e-8);
    }
}

#[test]
fn path_distance_and_location_are_bidirectional() {
    let path = curved_path();
    let index = PathMeasureIndex::build(&path, Tolerance::default()).unwrap();

    for (segment_index, t) in [(0, 0.2), (0, 0.7), (1, 0.4), (1, 0.9)] {
        let distance = index.distance_at_location(0, segment_index, t).unwrap();
        let location = index.location_at_distance(distance).unwrap();

        assert_eq!(location.subpath_index, 0);
        assert_eq!(location.segment_index, segment_index);
        assert_close(location.t, t, 2.0e-7);
    }
}

#[test]
fn indexed_frame_matches_direct_frame() {
    let path = curved_path();
    let tolerance = Tolerance::default();
    let index = PathMeasureIndex::build(&path, tolerance).unwrap();
    let distance = index.total_length() * 0.42;

    let direct = path_frame_at_length(&path, distance, tolerance).unwrap();
    let indexed = path_frame_at_length_indexed(&index, distance).unwrap();

    assert!(direct.point.distance_to(indexed.point) <= 2.0e-6);
    assert!(direct.tangent.x.is_finite() && indexed.tangent.x.is_finite());
    assert_close(direct.curvature, indexed.curvature, 2.0e-6);
}

#[test]
fn retraced_collinear_cubic_builds_monotonic_measure_table() {
    let segment = Segment::Cubic(CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(18.0, 0.0),
        Point2::new(-8.0, 0.0),
        Point2::new(10.0, 0.0),
    ));
    let table = SegmentMeasureTable::build(segment, Tolerance::default()).unwrap();

    let mut previous = 0.0;
    for t in [0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
        let distance = table.length_at_parameter(t).unwrap();
        assert!(distance >= previous);
        previous = distance;
    }
    assert_close(previous, table.total_length(), 1.0e-9);
}

#[test]
fn measure_index_is_stable_under_large_translation() {
    let offset = 1.0e12;
    let segment = Segment::Cubic(CubicBezier::new(
        Point2::new(offset, offset),
        Point2::new(offset + 20.0, offset + 80.0),
        Point2::new(offset + 80.0, offset - 60.0),
        Point2::new(offset + 120.0, offset + 20.0),
    ));
    let table = SegmentMeasureTable::build(segment, Tolerance::default()).unwrap();

    let distance = table.length_at_parameter(0.63).unwrap();
    let recovered = table.parameter_at_length(distance).unwrap();
    assert_close(recovered, 0.63, 2.0e-7);

    for fraction in [0.2, 0.5, 0.8] {
        let wanted = table.total_length() * fraction;
        let indexed = table.point_at_length(wanted).unwrap();
        let exact_t = segment_parameter_at_length(segment, wanted, Tolerance::default()).unwrap();
        let exact = segment.point_at(exact_t);
        assert!(indexed.distance_to(exact) <= 0.01);
    }
}

#[test]
fn measure_index_rejects_invalid_queries() {
    let path = curved_path();
    let index = PathMeasureIndex::build(&path, Tolerance::default()).unwrap();

    assert_eq!(
        index.location_at_distance(f64::NAN),
        Err(CoreError::InvalidNumber)
    );
    assert_eq!(
        index.distance_at_location(0, 0, 1.5),
        Err(CoreError::InvalidGeometry)
    );
    assert_eq!(
        index.distance_at_location(4, 0, 0.5),
        Err(CoreError::InvalidGeometry)
    );
}

#[test]
fn empty_path_index_has_defined_query_failure() {
    let index = PathMeasureIndex::build(&Path::new(), Tolerance::default()).unwrap();

    assert_eq!(index.total_length(), 0.0);
    assert_eq!(
        index.location_at_distance(0.0),
        Err(CoreError::DegenerateOperation)
    );
}
