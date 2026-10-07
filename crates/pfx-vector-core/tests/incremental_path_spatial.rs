use pfx_vector_core::*;

fn base_path() -> Path {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(20.0, 40.0),
            Point2::new(40.0, 40.0),
            Point2::new(60.0, 0.0),
        )
        .unwrap()
        .line_to(Point2::new(60.0, 60.0))
        .unwrap()
        .line_to(Point2::new(0.0, 60.0))
        .unwrap()
        .close()
        .unwrap()
        .move_to(Point2::new(100.0, 0.0))
        .unwrap()
        .line_to(Point2::new(120.0, 0.0))
        .unwrap()
        .line_to(Point2::new(120.0, 20.0))
        .unwrap();

    builder.finish().unwrap()
}

fn changed_curve_path() -> Path {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(15.0, 55.0),
            Point2::new(45.0, 55.0),
            Point2::new(60.0, 0.0),
        )
        .unwrap()
        .line_to(Point2::new(60.0, 60.0))
        .unwrap()
        .line_to(Point2::new(0.0, 60.0))
        .unwrap()
        .close()
        .unwrap()
        .move_to(Point2::new(100.0, 0.0))
        .unwrap()
        .line_to(Point2::new(120.0, 0.0))
        .unwrap()
        .line_to(Point2::new(120.0, 20.0))
        .unwrap();

    builder.finish().unwrap()
}

fn topology_changed_path() -> Path {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(30.0, 0.0))
        .unwrap()
        .line_to(Point2::new(60.0, 0.0))
        .unwrap()
        .line_to(Point2::new(60.0, 60.0))
        .unwrap()
        .line_to(Point2::new(0.0, 60.0))
        .unwrap()
        .close()
        .unwrap()
        .move_to(Point2::new(100.0, 0.0))
        .unwrap()
        .line_to(Point2::new(120.0, 0.0))
        .unwrap()
        .line_to(Point2::new(120.0, 20.0))
        .unwrap();

    builder.finish().unwrap()
}

fn single_subpath_path() -> Path {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 10.0))
        .unwrap();

    builder.finish().unwrap()
}

fn two_subpath_path() -> Path {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap()
        .move_to(Point2::new(20.0, 0.0))
        .unwrap()
        .line_to(Point2::new(30.0, 0.0))
        .unwrap();

    builder.finish().unwrap()
}

fn assert_queries_match(path: &Path, index: &IncrementalPathSpatialIndex, tolerance: Tolerance) {
    let style = StrokeStyle {
        width: 4.0,
        ..StrokeStyle::default()
    };

    for point in [
        Point2::new(-5.0, -5.0),
        Point2::new(5.0, 5.0),
        Point2::new(30.0, 20.0),
        Point2::new(59.0, 30.0),
        Point2::new(90.0, 10.0),
        Point2::new(120.0, 10.0),
    ] {
        assert_eq!(
            index.contains_point(point, FillRule::NonZero).unwrap(),
            contains_point(path, point, FillRule::NonZero, tolerance).unwrap(),
            "fill mismatch at {point:?}"
        );
        assert_eq!(
            index.stroke_contains_point(&style, point).unwrap(),
            stroke_contains_point(path, &style, point, tolerance).unwrap(),
            "stroke mismatch at {point:?}"
        );
    }

    for point in [
        Point2::new(30.0, 70.0),
        Point2::new(70.0, 10.0),
        Point2::new(115.0, 8.0),
    ] {
        let indexed = index.closest_point(point).unwrap();
        let direct = closest_point(path, point, tolerance).unwrap();
        assert!(
            indexed.point.distance_to(direct.point) <= tolerance.flatness * 2.0,
            "closest point mismatch at {point:?}: indexed={:?}, direct={:?}",
            indexed.point,
            direct.point
        );
        assert!(
            (indexed.distance - direct.distance).abs() <= tolerance.flatness * 2.0,
            "closest distance mismatch at {point:?}"
        );
    }
}

#[test]
fn incremental_index_matches_full_algorithms_before_and_after_sync() {
    let tolerance = Tolerance {
        flatness: 1.0e-3,
        ..Tolerance::default()
    };
    let original = base_path();
    let changed = changed_curve_path();
    let mut index = IncrementalPathSpatialIndex::build(&original, tolerance).unwrap();

    assert_queries_match(&original, &index, tolerance);
    assert!(
        (index.total_length() - path_length(&original, tolerance).unwrap()).abs()
            <= tolerance.absolute
    );

    let report = index.sync_path(&changed).unwrap();
    assert!(!report.full_rebuild);
    assert_eq!(report.changed_subpaths, 1);
    assert_eq!(report.changed_segments, 1);
    assert!(report.inserted_edges > 0);
    assert!(report.removed_edges > 0);

    assert_queries_match(&changed, &index, tolerance);
    assert!(
        (index.total_length() - path_length(&changed, tolerance).unwrap()).abs()
            <= tolerance.absolute
    );
    assert_eq!(index.path(), &changed);
}

#[test]
fn identical_sync_is_a_no_op() {
    let tolerance = Tolerance::default();
    let path = base_path();
    let mut index = IncrementalPathSpatialIndex::build(&path, tolerance).unwrap();
    let edge_count = index.edge_count();

    let report = index.sync_path(&path).unwrap();

    assert_eq!(report, PathSpatialSync::default());
    assert_eq!(index.edge_count(), edge_count);
}

#[test]
fn segment_count_change_rebuilds_only_the_changed_subpath() {
    let tolerance = Tolerance::default();
    let original = base_path();
    let changed = topology_changed_path();
    let mut index = IncrementalPathSpatialIndex::build(&original, tolerance).unwrap();

    let report = index.sync_path(&changed).unwrap();

    assert!(!report.full_rebuild);
    assert_eq!(report.changed_subpaths, 1);
    assert!(report.changed_segments >= 4);
    assert_eq!(index.segment_count(), changed.segment_count());
    assert_queries_match(&changed, &index, tolerance);
}

#[test]
fn subpath_count_change_uses_full_rebuild() {
    let tolerance = Tolerance::default();
    let original = single_subpath_path();
    let changed = two_subpath_path();
    let mut index = IncrementalPathSpatialIndex::build(&original, tolerance).unwrap();

    let report = index.sync_path(&changed).unwrap();

    assert!(report.full_rebuild);
    assert_eq!(report.changed_subpaths, 2);
    assert_eq!(index.path(), &changed);
    assert_eq!(index.segment_count(), changed.segment_count());
}

#[test]
fn closest_location_uses_source_segment_coordinates() {
    let tolerance = Tolerance::default();
    let path = single_subpath_path();
    let index = IncrementalPathSpatialIndex::build(&path, tolerance).unwrap();

    let result = index.closest_point(Point2::new(12.0, 5.0)).unwrap();

    assert_eq!(result.location.subpath_index, 0);
    assert_eq!(result.location.segment_index, 1);
    assert!((result.location.t - 0.5).abs() <= 1.0e-12);
    assert!((result.location.distance - 15.0).abs() <= 1.0e-12);
    assert_eq!(result.point, Point2::new(10.0, 5.0));
}

#[test]
fn repeated_sync_cycles_remain_deterministic() {
    let tolerance = Tolerance {
        flatness: 1.0e-3,
        ..Tolerance::default()
    };
    let original = base_path();
    let changed = changed_curve_path();
    let mut index = IncrementalPathSpatialIndex::build(&original, tolerance).unwrap();

    for _ in 0..20 {
        let report_changed = index.sync_path(&changed).unwrap();
        assert!(!report_changed.full_rebuild);
        let changed_result = index.closest_point(Point2::new(30.0, 70.0)).unwrap();

        let report_original = index.sync_path(&original).unwrap();
        assert!(!report_original.full_rebuild);
        let original_result = index.closest_point(Point2::new(30.0, 70.0)).unwrap();

        assert!(changed_result.distance.is_finite());
        assert!(original_result.distance.is_finite());
        assert_queries_match(&original, &index, tolerance);
    }
}

#[test]
fn incremental_index_is_stable_at_large_coordinates() {
    let tolerance = Tolerance {
        flatness: 1.0e-3,
        ..Tolerance::default()
    };
    let offset = 1.0e12;

    let mut original_builder = PathBuilder::new();
    original_builder
        .move_to(Point2::new(offset, offset))
        .unwrap()
        .line_to(Point2::new(offset + 20.0, offset))
        .unwrap()
        .line_to(Point2::new(offset + 20.0, offset + 20.0))
        .unwrap();
    let original = original_builder.finish().unwrap();

    let mut changed_builder = PathBuilder::new();
    changed_builder
        .move_to(Point2::new(offset, offset))
        .unwrap()
        .line_to(Point2::new(offset + 20.0, offset))
        .unwrap()
        .line_to(Point2::new(offset + 25.0, offset + 20.0))
        .unwrap();
    let changed = changed_builder.finish().unwrap();

    let mut index = IncrementalPathSpatialIndex::build(&original, tolerance).unwrap();
    let report = index.sync_path(&changed).unwrap();

    assert!(!report.full_rebuild);
    assert_eq!(report.changed_segments, 1);

    let point = Point2::new(offset + 24.0, offset + 10.0);
    let indexed = index.closest_point(point).unwrap();
    let direct = closest_point(&changed, point, tolerance).unwrap();
    assert!(indexed.point.distance_to(direct.point) <= 1.0e-3);
}

#[test]
fn incremental_nearest_refines_against_source_curve_after_sync() {
    let tolerance = Tolerance {
        flatness: 100.0,
        ..Tolerance::default()
    };
    let mut first_builder = PathBuilder::new();
    first_builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .quad_to(Point2::new(5.0, 6.0), Point2::new(10.0, 0.0))
        .unwrap();
    let first = first_builder.finish().unwrap();

    let mut second_builder = PathBuilder::new();
    second_builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .quad_to(Point2::new(5.0, 10.0), Point2::new(10.0, 0.0))
        .unwrap();
    let second = second_builder.finish().unwrap();

    let mut index = IncrementalPathSpatialIndex::build(&first, tolerance).unwrap();
    index.sync_path(&second).unwrap();
    let result = index.closest_point(Point2::new(5.0, 8.0)).unwrap();

    assert_eq!(result.location.segment_index, 0);
    assert!((result.location.t - 0.5).abs() <= 1.0e-9);
    assert!(result.point.distance_to(Point2::new(5.0, 5.0)) <= 1.0e-8);
    assert!((result.distance - 3.0).abs() <= 1.0e-8);
}

#[test]
fn incremental_fill_refines_against_source_curve_after_sync() {
    let tolerance = Tolerance {
        flatness: 1000.0,
        ..Tolerance::default()
    };

    let mut first_builder = PathBuilder::new();
    first_builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(0.0, 6.0),
            Point2::new(10.0, 6.0),
            Point2::new(10.0, 0.0),
        )
        .unwrap()
        .close()
        .unwrap();
    let first = first_builder.finish().unwrap();

    let mut second_builder = PathBuilder::new();
    second_builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(0.0, 10.0),
            Point2::new(10.0, 10.0),
            Point2::new(10.0, 0.0),
        )
        .unwrap()
        .close()
        .unwrap();
    let second = second_builder.finish().unwrap();

    let mut index = IncrementalPathSpatialIndex::build(&first, tolerance).unwrap();
    index.sync_path(&second).unwrap();

    assert_eq!(
        index
            .classify_point(Point2::new(5.0, 4.0), FillRule::NonZero)
            .unwrap(),
        PointClassification::Inside
    );
    assert_eq!(
        index
            .classify_point(Point2::new(5.0, 8.0), FillRule::NonZero)
            .unwrap(),
        PointClassification::Outside
    );
}
