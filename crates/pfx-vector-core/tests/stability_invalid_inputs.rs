//! Regressions for malformed numerical inputs and path boundaries.
use pfx_vector_core::*;

fn line_index() -> PathMeasureIndex {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(3.0, 4.0))
        .unwrap();
    PathMeasureIndex::build(&builder.finish().unwrap(), Tolerance::default()).unwrap()
}

#[test]
fn measurement_rejects_non_finite_distance_queries() {
    let index = line_index();
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(index.location_at_distance(invalid), Err(CoreError::InvalidNumber));
        assert_eq!(index.point_at_length(invalid), Err(CoreError::InvalidNumber));
    }
}

#[test]
fn measurement_rejects_non_finite_and_out_of_range_parameters() {
    let index = line_index();
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            index.distance_at_location(0, 0, invalid),
            Err(CoreError::InvalidNumber)
        );
    }
    for invalid in [-0.01, 1.01] {
        assert_eq!(
            index.distance_at_location(0, 0, invalid),
            Err(CoreError::InvalidGeometry)
        );
    }
}

#[test]
fn measurement_rejects_out_of_range_source_addresses() {
    let index = line_index();
    for (subpath, segment) in [(0, 1), (1, 0), (usize::MAX, 0), (0, usize::MAX)] {
        assert_eq!(
            index.segment_table(subpath, segment).map(|_| ()),
            Err(CoreError::InvalidGeometry)
        );
    }
}

#[test]
fn line_distance_queries_are_clamped_and_finite() {
    let index = line_index();
    for (distance, expected) in [(-100.0, 0.0), (0.0, 0.0), (2.5, 2.5), (5.0, 5.0), (100.0, 5.0)] {
        let (point, location) = index.point_at_length(distance).unwrap();
        assert_eq!(location.distance, expected);
        assert!(point.x.is_finite() && point.y.is_finite());
        assert!((point.distance_to(Point2::new(expected * 0.6, expected * 0.8))) <= 1.0e-12);
    }
}

#[test]
fn non_finite_path_builder_inputs_do_not_contaminate_geometry() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut builder = PathBuilder::new();
        assert_eq!(
            builder.move_to(Point2::new(bad, 0.0)).map(|_| ()),
            Err(CoreError::InvalidNumber)
        );
        builder.move_to(Point2::new(0.0, 0.0)).unwrap();
        assert_eq!(
            builder.line_to(Point2::new(bad, 1.0)).map(|_| ()),
            Err(CoreError::InvalidNumber)
        );
        builder.line_to(Point2::new(3.0, 4.0)).unwrap();
        let path = builder.finish().unwrap();
        assert_eq!(path.segment_count(), 1);
    }
}
