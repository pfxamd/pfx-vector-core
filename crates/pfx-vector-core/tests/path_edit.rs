use pfx_vector_core::*;

fn line_path(points: &[Point2]) -> Path {
    let mut builder = PathBuilder::new();
    if let Some(&first) = points.first() {
        builder.move_to(first).unwrap();
        for &point in &points[1..] {
            builder.line_to(point).unwrap();
        }
    }
    builder.finish().unwrap()
}

fn only_subpath(path: &Path) -> &Subpath {
    assert_eq!(path.subpaths().len(), 1);
    &path.subpaths()[0]
}

#[test]
fn split_line_inserts_anchor_and_reports_topology_change() {
    let path = line_path(&[Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)]);
    let tolerance = Tolerance::default();
    let edit = split_segment(&path, SegmentAddress::new(0, 0), 0.25, tolerance).unwrap();

    let subpath = only_subpath(edit.path());
    assert_eq!(subpath.segments().len(), 2);
    assert!(
        subpath.segments()[0]
            .end()
            .almost_eq(Point2::new(2.5, 0.0), tolerance)
    );
    assert!(
        subpath.segments()[1]
            .start()
            .almost_eq(Point2::new(2.5, 0.0), tolerance)
    );
    assert_eq!(edit.report().kind, PathEditKind::SplitSegment);
    assert_eq!(edit.report().segments_before, 1);
    assert_eq!(edit.report().segments_after, 2);
    assert_eq!(edit.report().subpath_delta, 0);
    assert_eq!(edit.report().segment_delta, 1);
    assert!(edit.report().topology_changed);
}

#[test]
fn split_preserves_quadratic_cubic_and_arc_primitives() {
    let tolerance = Tolerance::default();

    let quadratic = QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(5.0, 10.0),
        Point2::new(10.0, 0.0),
    );
    let mut quadratic_builder = PathBuilder::new();
    quadratic_builder
        .move_to(quadratic.p0)
        .unwrap()
        .quad_to(quadratic.p1, quadratic.p2)
        .unwrap();
    let quadratic_path = quadratic_builder.finish().unwrap();
    let quadratic_edit =
        split_segment(&quadratic_path, SegmentAddress::new(0, 0), 0.4, tolerance).unwrap();
    let quadratic_segments = only_subpath(quadratic_edit.path()).segments();
    assert!(matches!(quadratic_segments[0], Segment::Quadratic(_)));
    assert!(matches!(quadratic_segments[1], Segment::Quadratic(_)));
    assert!(
        quadratic_segments[0]
            .end()
            .almost_eq(quadratic.point_at(0.4), tolerance)
    );

    let cubic = CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(20.0, 40.0),
        Point2::new(80.0, -40.0),
        Point2::new(100.0, 0.0),
    );
    let mut cubic_builder = PathBuilder::new();
    cubic_builder
        .move_to(cubic.p0)
        .unwrap()
        .cubic_to(cubic.p1, cubic.p2, cubic.p3)
        .unwrap();
    let cubic_path = cubic_builder.finish().unwrap();
    let cubic_edit = split_segment(&cubic_path, SegmentAddress::new(0, 0), 0.6, tolerance).unwrap();
    let cubic_segments = only_subpath(cubic_edit.path()).segments();
    assert!(matches!(cubic_segments[0], Segment::Cubic(_)));
    assert!(matches!(cubic_segments[1], Segment::Cubic(_)));
    assert!(
        cubic_segments[0]
            .end()
            .almost_eq(cubic.point_at(0.6), tolerance)
    );

    let arc = EllipticalArc::new(
        Point2::new(0.0, 0.0),
        30.0,
        20.0,
        Angle::radians(0.2),
        Angle::radians(0.1),
        Angle::radians(core::f64::consts::PI),
    );
    let mut arc_builder = PathBuilder::new();
    arc_builder
        .move_to(arc.point_at(0.0))
        .unwrap()
        .arc_to(arc)
        .unwrap();
    let arc_path = arc_builder.finish().unwrap();
    let arc_edit = split_segment(&arc_path, SegmentAddress::new(0, 0), 0.3, tolerance).unwrap();
    let arc_segments = only_subpath(arc_edit.path()).segments();
    assert!(matches!(arc_segments[0], Segment::Arc(_)));
    assert!(matches!(arc_segments[1], Segment::Arc(_)));
    assert!(
        arc_segments[0]
            .end()
            .almost_eq(arc.point_at(0.3), tolerance)
    );
}

#[test]
fn split_segment_at_length_uses_arc_length() {
    let path = line_path(&[Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)]);
    let tolerance = Tolerance::default();

    let edit = split_segment_at_length(&path, SegmentAddress::new(0, 0), 7.5, tolerance).unwrap();

    assert!(
        only_subpath(edit.path()).segments()[0]
            .end()
            .almost_eq(Point2::new(7.5, 0.0), tolerance)
    );
}

#[test]
fn insert_anchor_matches_segment_split() {
    let path = line_path(&[Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)]);
    let tolerance = Tolerance::default();

    let split = split_segment(&path, SegmentAddress::new(0, 0), 0.5, tolerance).unwrap();
    let anchor = insert_anchor(&path, SegmentAddress::new(0, 0), 0.5, tolerance).unwrap();

    assert_eq!(split.path, anchor.path);
}

#[test]
fn reverse_subpath_twice_restores_original_geometry() {
    let path = line_path(&[
        Point2::new(0.0, 0.0),
        Point2::new(10.0, 5.0),
        Point2::new(20.0, 0.0),
    ]);

    let reversed = reverse_subpath(&path, 0).unwrap();
    assert_eq!(
        only_subpath(reversed.path()).start(),
        Point2::new(20.0, 0.0)
    );
    assert_eq!(only_subpath(reversed.path()).end(), Point2::new(0.0, 0.0));

    let restored = reverse_subpath(reversed.path(), 0).unwrap();
    assert_eq!(restored.path, path);
}

#[test]
fn open_and_close_contour_preserve_stored_segments() {
    let path = line_path(&[
        Point2::new(0.0, 0.0),
        Point2::new(10.0, 0.0),
        Point2::new(10.0, 10.0),
    ]);
    let tolerance = Tolerance::default();

    let closed = set_subpath_closed(&path, 0, true, tolerance).unwrap();
    assert!(only_subpath(closed.path()).is_closed());
    assert_eq!(
        only_subpath(closed.path()).segments(),
        only_subpath(&path).segments()
    );

    let opened = set_subpath_closed(closed.path(), 0, false, tolerance).unwrap();
    assert_eq!(opened.path, path);
}

#[test]
fn join_subpaths_matches_selected_endpoints() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap()
        .move_to(Point2::new(20.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap();
    let path = builder.finish().unwrap();
    let tolerance = Tolerance::default();

    let joined = join_open_subpaths(
        &path,
        0,
        SubpathEndpoint::End,
        1,
        SubpathEndpoint::End,
        tolerance,
    )
    .unwrap();

    let subpath = only_subpath(joined.path());
    assert_eq!(subpath.segments().len(), 2);
    assert!(subpath.start().almost_eq(Point2::new(0.0, 0.0), tolerance));
    assert!(subpath.end().almost_eq(Point2::new(20.0, 0.0), tolerance));
    assert_eq!(joined.report().subpaths_before, 2);
    assert_eq!(joined.report().subpaths_after, 1);
}

#[test]
fn join_rejects_non_matching_endpoints() {
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
    let path = builder.finish().unwrap();

    assert_eq!(
        join_open_subpaths(
            &path,
            0,
            SubpathEndpoint::End,
            1,
            SubpathEndpoint::Start,
            Tolerance::default(),
        ),
        Err(CoreError::InvalidGeometry)
    );
}

#[test]
fn removing_middle_open_segment_splits_subpath() {
    let path = line_path(&[
        Point2::new(0.0, 0.0),
        Point2::new(10.0, 0.0),
        Point2::new(20.0, 0.0),
        Point2::new(30.0, 0.0),
    ]);

    let edit = remove_segment(&path, SegmentAddress::new(0, 1), Tolerance::default()).unwrap();

    assert_eq!(edit.path.subpaths().len(), 2);
    assert_eq!(edit.report().subpath_delta, 1);
    assert_eq!(edit.report().segment_delta, -1);
    assert_eq!(edit.path.subpaths()[0].start(), Point2::new(0.0, 0.0));
    assert_eq!(edit.path.subpaths()[0].end(), Point2::new(10.0, 0.0));
    assert_eq!(edit.path.subpaths()[1].start(), Point2::new(20.0, 0.0));
    assert_eq!(edit.path.subpaths()[1].end(), Point2::new(30.0, 0.0));
}

#[test]
fn removing_segment_from_closed_contour_opens_at_removed_edge() {
    let path = rect_to_path(Rect::new(0.0, 0.0, 10.0, 10.0).unwrap()).unwrap();
    let tolerance = Tolerance::default();

    let edit = remove_segment(&path, SegmentAddress::new(0, 1), tolerance).unwrap();
    let subpath = only_subpath(edit.path());

    assert!(!subpath.is_closed());
    assert_eq!(subpath.segments().len(), 3);
    assert!((path_length(edit.path(), tolerance).unwrap() - 30.0).abs() < 1.0e-8);
}

#[test]
fn replace_segment_preserves_endpoints_and_changes_primitive() {
    let path = line_path(&[Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)]);
    let replacement = Segment::Quadratic(QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(5.0, 5.0),
        Point2::new(10.0, 0.0),
    ));

    let edit = replace_segment(
        &path,
        SegmentAddress::new(0, 0),
        replacement,
        Tolerance::default(),
    )
    .unwrap();

    assert!(matches!(
        only_subpath(edit.path()).segments()[0],
        Segment::Quadratic(_)
    ));
    assert!(!edit.report().topology_changed);
}

#[test]
fn replace_segment_rejects_endpoint_mismatch() {
    let path = line_path(&[Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)]);
    let replacement = Segment::Line(LineSegment::new(
        Point2::new(1.0, 0.0),
        Point2::new(10.0, 0.0),
    ));

    assert_eq!(
        replace_segment(
            &path,
            SegmentAddress::new(0, 0),
            replacement,
            Tolerance::default(),
        ),
        Err(CoreError::InvalidGeometry)
    );
}

#[test]
fn remove_subpath_keeps_other_subpaths_in_order() {
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
    let path = builder.finish().unwrap();

    let edit = remove_subpath(&path, 0).unwrap();

    assert_eq!(edit.path.subpaths().len(), 1);
    assert_eq!(edit.path.subpaths()[0].start(), Point2::new(20.0, 0.0));
    assert_eq!(edit.report().affected_subpaths, vec![0]);
}

#[test]
fn incremental_spatial_index_can_sync_edit_result() {
    let path = line_path(&[
        Point2::new(0.0, 0.0),
        Point2::new(10.0, 0.0),
        Point2::new(20.0, 0.0),
    ]);
    let tolerance = Tolerance::default();
    let mut index = IncrementalPathSpatialIndex::build(&path, tolerance).unwrap();

    let edit = split_segment(&path, SegmentAddress::new(0, 0), 0.5, tolerance).unwrap();
    let sync = index.sync_edit(&edit).unwrap();

    assert_eq!(index.path(), edit.path());
    assert!(!sync.full_rebuild);
    assert_eq!(sync.changed_subpaths, 1);
    assert!((index.total_length() - 20.0).abs() < 1.0e-8);
}

#[test]
fn large_coordinate_split_remains_stable() {
    let base = 1.0e9;
    let path = line_path(&[Point2::new(base, base), Point2::new(base + 100.0, base)]);
    let tolerance = Tolerance::default();

    let edit = split_segment(&path, SegmentAddress::new(0, 0), 0.5, tolerance).unwrap();
    let anchor = only_subpath(edit.path()).segments()[0].end();

    assert!((anchor.x - (base + 50.0)).abs() < 1.0e-6);
    assert!((anchor.y - base).abs() < 1.0e-6);
}

#[test]
fn invalid_split_parameter_and_address_are_rejected() {
    let path = line_path(&[Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)]);

    assert_eq!(
        split_segment(&path, SegmentAddress::new(0, 0), 0.0, Tolerance::default(),),
        Err(CoreError::InvalidGeometry)
    );
    assert_eq!(
        split_segment(&path, SegmentAddress::new(0, 5), 0.5, Tolerance::default(),),
        Err(CoreError::InvalidGeometry)
    );
}


#[test]
fn extract_segment_and_subpath_return_source_geometry() {
    let path = line_path(&[
        Point2::new(0.0, 0.0),
        Point2::new(10.0, 0.0),
        Point2::new(20.0, 5.0),
    ]);

    let segment = extract_segment(&path, SegmentAddress::new(0, 1)).unwrap();
    assert_eq!(segment, path.subpaths()[0].segments()[1]);

    let subpath = extract_subpath(&path, 0).unwrap();
    assert_eq!(subpath, path.subpaths()[0]);
}

#[test]
fn replace_subpath_updates_selected_contour_and_report() {
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
    let path = builder.finish().unwrap();

    let replacement_path = line_path(&[
        Point2::new(100.0, 0.0),
        Point2::new(110.0, 10.0),
        Point2::new(120.0, 0.0),
    ]);
    let replacement = extract_subpath(&replacement_path, 0).unwrap();

    let edit = replace_subpath(&path, 1, replacement.clone()).unwrap();

    assert_eq!(edit.path.subpaths()[0], path.subpaths()[0]);
    assert_eq!(edit.path.subpaths()[1], replacement);
    assert_eq!(edit.report().kind, PathEditKind::ReplaceSubpath);
    assert_eq!(edit.report().affected_subpaths, vec![1]);
    assert_eq!(edit.report().subpath_delta, 0);
    assert_eq!(edit.report().segment_delta, 1);
    assert!(edit.report().topology_changed);
}

#[test]
fn replace_subpath_same_topology_reports_no_index_shift() {
    let path = line_path(&[Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)]);
    let replacement_path =
        line_path(&[Point2::new(0.0, 0.0), Point2::new(10.0, 5.0)]);
    let replacement = extract_subpath(&replacement_path, 0).unwrap();

    let edit = replace_subpath(&path, 0, replacement).unwrap();

    assert_eq!(edit.report().subpath_delta, 0);
    assert_eq!(edit.report().segment_delta, 0);
    assert!(!edit.report().topology_changed);
}
