use pfx_vector_core::*;

fn open_polyline(points: &[Point2]) -> Path {
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
fn signed_open_line_offset_uses_left_and_right_sides() {
    let path = open_polyline(&[Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)]);
    let tolerance = Tolerance::default();

    let left = offset_path(&path, 2.0, OffsetStyle::default(), tolerance).unwrap();
    let right = offset_path(&path, -3.0, OffsetStyle::default(), tolerance).unwrap();

    assert!(
        only_subpath(&left)
            .start()
            .almost_eq(Point2::new(0.0, 2.0), tolerance)
    );
    assert!(
        only_subpath(&left)
            .end()
            .almost_eq(Point2::new(10.0, 2.0), tolerance)
    );
    assert!(
        only_subpath(&right)
            .start()
            .almost_eq(Point2::new(0.0, -3.0), tolerance)
    );
    assert!(
        only_subpath(&right)
            .end()
            .almost_eq(Point2::new(10.0, -3.0), tolerance)
    );
}

#[test]
fn miter_join_is_exact_for_line_corner_on_both_sides() {
    let path = open_polyline(&[
        Point2::new(0.0, 0.0),
        Point2::new(10.0, 0.0),
        Point2::new(10.0, 10.0),
    ]);
    let tolerance = Tolerance::default();
    let style = OffsetStyle {
        join: StrokeJoin::Miter,
        miter_limit: 4.0,
    };

    let inside = offset_path(&path, 2.0, style, tolerance).unwrap();
    let outside = offset_path(&path, -2.0, style, tolerance).unwrap();

    let inside_segments = only_subpath(&inside).segments();
    assert_eq!(inside_segments.len(), 2);
    assert!(
        inside_segments[0]
            .end()
            .almost_eq(Point2::new(8.0, 2.0), tolerance)
    );
    assert!(
        inside_segments[1]
            .start()
            .almost_eq(Point2::new(8.0, 2.0), tolerance)
    );

    let outside_segments = only_subpath(&outside).segments();
    assert_eq!(outside_segments.len(), 2);
    assert!(
        outside_segments[0]
            .end()
            .almost_eq(Point2::new(12.0, -2.0), tolerance)
    );
    assert!(
        outside_segments[1]
            .start()
            .almost_eq(Point2::new(12.0, -2.0), tolerance)
    );
}

#[test]
fn bevel_join_connects_outer_corner_without_miter_extension() {
    let path = open_polyline(&[
        Point2::new(0.0, 0.0),
        Point2::new(10.0, 0.0),
        Point2::new(10.0, 10.0),
    ]);
    let tolerance = Tolerance::default();
    let result = offset_path(
        &path,
        -2.0,
        OffsetStyle {
            join: StrokeJoin::Bevel,
            miter_limit: 4.0,
        },
        tolerance,
    )
    .unwrap();

    let segments = only_subpath(&result).segments();
    assert_eq!(segments.len(), 3);
    assert!(matches!(segments[1], Segment::Line(_)));
    assert!(
        segments[0]
            .end()
            .almost_eq(Point2::new(10.0, -2.0), tolerance)
    );
    assert!(
        segments[1]
            .end()
            .almost_eq(Point2::new(12.0, 0.0), tolerance)
    );
}

#[test]
fn round_join_preserves_native_arc_at_outer_corner() {
    let path = open_polyline(&[
        Point2::new(0.0, 0.0),
        Point2::new(10.0, 0.0),
        Point2::new(10.0, 10.0),
    ]);
    let tolerance = Tolerance::default();
    let result = offset_path(
        &path,
        -2.0,
        OffsetStyle {
            join: StrokeJoin::Round,
            miter_limit: 4.0,
        },
        tolerance,
    )
    .unwrap();

    let segments = only_subpath(&result).segments();
    assert_eq!(segments.len(), 3);
    assert!(matches!(segments[1], Segment::Arc(_)));
    assert!(segments[0].end().almost_eq(segments[1].start(), tolerance));
    assert!(segments[1].end().almost_eq(segments[2].start(), tolerance));
}

#[test]
fn low_miter_limit_falls_back_to_bevel_on_outer_corner() {
    let path = open_polyline(&[
        Point2::new(0.0, 0.0),
        Point2::new(10.0, 0.0),
        Point2::new(10.0, 10.0),
    ]);
    let tolerance = Tolerance::default();
    let result = offset_path(
        &path,
        -2.0,
        OffsetStyle {
            join: StrokeJoin::Miter,
            miter_limit: 1.0,
        },
        tolerance,
    )
    .unwrap();

    let segments = only_subpath(&result).segments();
    assert_eq!(segments.len(), 3);
    assert!(segments[0].end().almost_eq(Point2::new(10.0, -2.0), tolerance));
    assert!(segments[1].end().almost_eq(Point2::new(12.0, 0.0), tolerance));
}

#[test]
fn open_cubic_offset_remains_cubic_geometry() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(25.0, 50.0),
            Point2::new(75.0, -50.0),
            Point2::new(100.0, 0.0),
        )
        .unwrap();
    let path = builder.finish().unwrap();

    let result = offset_path(&path, 3.0, OffsetStyle::default(), Tolerance::default()).unwrap();

    assert!(
        only_subpath(&result)
            .segments()
            .iter()
            .all(|segment| matches!(segment, Segment::Cubic(_)))
    );
}

#[test]
fn circular_arc_offset_preserves_arc_primitive() {
    let arc = EllipticalArc::new(
        Point2::new(0.0, 0.0),
        20.0,
        20.0,
        Angle::radians(0.0),
        Angle::radians(0.0),
        Angle::radians(core::f64::consts::FRAC_PI_2),
    );
    let mut builder = PathBuilder::new();
    builder
        .move_to(arc.point_at(0.0))
        .unwrap()
        .arc_to(arc)
        .unwrap();
    let path = builder.finish().unwrap();

    let result = offset_path(&path, 2.0, OffsetStyle::default(), Tolerance::default()).unwrap();

    assert_eq!(only_subpath(&result).segments().len(), 1);
    assert!(matches!(
        only_subpath(&result).segments()[0],
        Segment::Arc(_)
    ));
}

#[test]
fn open_offset_accepts_trimmed_contour_output() {
    let closed = rect_to_path(Rect::new(0.0, 0.0, 20.0, 10.0).unwrap()).unwrap();
    let tolerance = Tolerance::default();
    let contour = &closed.subpaths()[0];
    let trimmed = slice_contour(contour, 5.0, 25.0, ContourSliceMode::Clamp, tolerance).unwrap();

    let offset = offset_path(&trimmed, 2.0, OffsetStyle::default(), tolerance).unwrap();

    assert!(!offset.is_empty());
    assert!(!only_subpath(&offset).is_closed());
}

#[test]
fn mixed_closed_and_open_subpaths_are_both_preserved() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 10.0))
        .unwrap()
        .line_to(Point2::new(0.0, 10.0))
        .unwrap()
        .close()
        .unwrap()
        .move_to(Point2::new(20.0, 0.0))
        .unwrap()
        .line_to(Point2::new(30.0, 0.0))
        .unwrap();
    let path = builder.finish().unwrap();

    let result = offset_path(&path, 1.0, OffsetStyle::default(), Tolerance::default()).unwrap();

    assert!(result.subpaths().iter().any(Subpath::is_closed));
    assert!(result.subpaths().iter().any(|subpath| !subpath.is_closed()));
}

#[test]
fn large_coordinate_open_offset_remains_stable() {
    let base = 1.0e9;
    let path = open_polyline(&[
        Point2::new(base, base),
        Point2::new(base + 100.0, base),
        Point2::new(base + 100.0, base + 100.0),
    ]);
    let tolerance = Tolerance::default();

    let result = offset_path(&path, 5.0, OffsetStyle::default(), tolerance).unwrap();
    let subpath = only_subpath(&result);

    assert!(
        subpath
            .start()
            .almost_eq(Point2::new(base, base + 5.0), tolerance)
    );
    assert!(
        subpath
            .end()
            .almost_eq(Point2::new(base + 95.0, base + 100.0), tolerance)
    );
}

#[test]
fn zero_distance_open_offset_is_identity() {
    let path = open_polyline(&[
        Point2::new(0.0, 0.0),
        Point2::new(4.0, 2.0),
        Point2::new(8.0, 0.0),
    ]);

    let result = offset_path(&path, 0.0, OffsetStyle::default(), Tolerance::default()).unwrap();

    assert_eq!(result, path);
}
