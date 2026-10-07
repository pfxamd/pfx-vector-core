use pfx_vector_core::*;

fn rect_path(x: f64, y: f64, width: f64, height: f64) -> Path {
    rect_to_path(Rect::new(x, y, width, height).unwrap()).unwrap()
}

fn inside(path: &Path, x: f64, y: f64) -> bool {
    contains_point(
        path,
        Point2::new(x, y),
        FillRule::NonZero,
        Tolerance::default(),
    )
    .unwrap()
}

fn bounds_tuple(path: &Path) -> (f64, f64, f64, f64) {
    match path_bounds(path) {
        Bounds::Finite { min, max } => (min.x, min.y, max.x, max.y),
        Bounds::Empty => panic!("expected finite bounds"),
    }
}

#[test]
fn outward_rectangle_offset_expands_all_sides() {
    let path = rect_path(0.0, 0.0, 10.0, 10.0);
    let result = offset_path(&path, 2.0, OffsetStyle::default(), Tolerance::default()).unwrap();

    let bounds = bounds_tuple(&result);
    assert!((bounds.0 + 2.0).abs() < 1.0e-7);
    assert!((bounds.1 + 2.0).abs() < 1.0e-7);
    assert!((bounds.2 - 12.0).abs() < 1.0e-7);
    assert!((bounds.3 - 12.0).abs() < 1.0e-7);
    assert!(inside(&result, -1.0, 5.0));
    assert!(inside(&result, 11.0, 5.0));
}

#[test]
fn inward_rectangle_offset_contracts_shape() {
    let path = rect_path(0.0, 0.0, 10.0, 10.0);
    let result = offset_path(&path, -2.0, OffsetStyle::default(), Tolerance::default()).unwrap();

    let bounds = bounds_tuple(&result);
    assert!((bounds.0 - 2.0).abs() < 1.0e-7);
    assert!((bounds.1 - 2.0).abs() < 1.0e-7);
    assert!((bounds.2 - 8.0).abs() < 1.0e-7);
    assert!((bounds.3 - 8.0).abs() < 1.0e-7);
    assert!(inside(&result, 5.0, 5.0));
    assert!(!inside(&result, 1.0, 5.0));
}

#[test]
fn circle_offset_preserves_arc_geometry() {
    let circle = circle_to_path(Circle::new(Point2::new(0.0, 0.0), 10.0).unwrap()).unwrap();
    let result = offset_path(
        &circle,
        3.0,
        OffsetStyle {
            join: StrokeJoin::Round,
            miter_limit: 4.0,
        },
        Tolerance::default(),
    )
    .unwrap();

    let bounds = bounds_tuple(&result);
    assert!((bounds.0 + 13.0).abs() < 1.0e-7);
    assert!((bounds.2 - 13.0).abs() < 1.0e-7);
    assert!(
        result
            .subpaths()
            .iter()
            .flat_map(|subpath| subpath.segments())
            .any(|segment| matches!(segment, Segment::Arc(_)))
    );
}

#[test]
fn butt_outline_of_line_has_expected_extent() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap();
    let path = builder.finish().unwrap();

    let style = StrokeStyle {
        width: 4.0,
        cap: StrokeCap::Butt,
        join: StrokeJoin::Miter,
        miter_limit: 4.0,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };
    let result = outline_path(&path, &style, Tolerance::default()).unwrap();
    let bounds = bounds_tuple(&result);

    assert!((bounds.0 - 0.0).abs() < 1.0e-7);
    assert!((bounds.1 + 2.0).abs() < 1.0e-7);
    assert!((bounds.2 - 10.0).abs() < 1.0e-7);
    assert!((bounds.3 - 2.0).abs() < 1.0e-7);
}

#[test]
fn square_caps_extend_by_half_width() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap();
    let path = builder.finish().unwrap();

    let style = StrokeStyle {
        width: 4.0,
        cap: StrokeCap::Square,
        join: StrokeJoin::Miter,
        miter_limit: 4.0,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };
    let result = outline_path(&path, &style, Tolerance::default()).unwrap();
    let bounds = bounds_tuple(&result);

    assert!((bounds.0 + 2.0).abs() < 1.0e-7);
    assert!((bounds.2 - 12.0).abs() < 1.0e-7);
}

#[test]
fn round_caps_create_capsule() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap();
    let path = builder.finish().unwrap();

    let style = StrokeStyle {
        width: 4.0,
        cap: StrokeCap::Round,
        join: StrokeJoin::Round,
        miter_limit: 4.0,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };
    let result = outline_path(&path, &style, Tolerance::default()).unwrap();

    assert!(inside(&result, -1.5, 0.0));
    assert!(inside(&result, 11.5, 0.0));
    assert!(!inside(&result, -2.5, 0.0));
}

#[test]
fn round_join_fills_outer_corner() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 10.0))
        .unwrap();
    let path = builder.finish().unwrap();

    let style = StrokeStyle {
        width: 4.0,
        cap: StrokeCap::Butt,
        join: StrokeJoin::Round,
        miter_limit: 4.0,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };
    let result = outline_path(&path, &style, Tolerance::default()).unwrap();

    assert!(inside(&result, 11.0, -1.0));
    assert!(!inside(&result, 12.5, -2.5));
}

#[test]
fn miter_limit_falls_back_to_bevel() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap()
        .line_to(Point2::new(9.0, 0.1))
        .unwrap();
    let path = builder.finish().unwrap();

    let style = StrokeStyle {
        width: 2.0,
        cap: StrokeCap::Butt,
        join: StrokeJoin::Miter,
        miter_limit: 1.5,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };
    let result = outline_path(&path, &style, Tolerance::default()).unwrap();

    let bounds = bounds_tuple(&result);
    assert!(bounds.2 < 12.0);
}

#[test]
fn cubic_outline_uses_cubic_offset_approximation() {
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

    let style = StrokeStyle {
        width: 4.0,
        cap: StrokeCap::Round,
        join: StrokeJoin::Round,
        miter_limit: 4.0,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };
    let result = outline_path(&path, &style, Tolerance::default()).unwrap();

    assert!(
        result
            .subpaths()
            .iter()
            .flat_map(|subpath| subpath.segments())
            .any(|segment| matches!(segment, Segment::Cubic(_)))
    );
}

#[test]
fn dashed_outline_is_explicitly_deferred() {
    let path = rect_path(0.0, 0.0, 10.0, 10.0);
    let style = StrokeStyle {
        width: 2.0,
        dash_array: vec![2.0, 2.0],
        ..StrokeStyle::default()
    };

    assert_eq!(
        outline_path(&path, &style, Tolerance::default()),
        Err(CoreError::UnsupportedCase)
    );
}

#[test]
fn open_path_offset_is_rejected() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap();
    let path = builder.finish().unwrap();

    assert_eq!(
        offset_path(&path, 2.0, OffsetStyle::default(), Tolerance::default(),),
        Err(CoreError::UnsupportedCase)
    );
}

#[test]
fn closed_rectangle_outline_is_a_ring() {
    let path = rect_path(0.0, 0.0, 20.0, 20.0);
    let style = StrokeStyle {
        width: 4.0,
        cap: StrokeCap::Butt,
        join: StrokeJoin::Miter,
        miter_limit: 4.0,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };

    let result = outline_path(&path, &style, Tolerance::default()).unwrap();

    assert!(inside(&result, -1.0, 10.0));
    assert!(inside(&result, 1.0, 10.0));
    assert!(!inside(&result, 10.0, 10.0));
    assert!(!inside(&result, -3.0, 10.0));
}

#[test]
fn offset_updates_outer_and_hole_boundaries() {
    let outer = rect_path(0.0, 0.0, 30.0, 30.0);
    let inner = rect_path(10.0, 10.0, 10.0, 10.0);
    let ring = boolean_difference(&outer, &inner, Tolerance::default()).unwrap();

    let expanded = offset_path(&ring, 2.0, OffsetStyle::default(), Tolerance::default()).unwrap();
    let contracted =
        offset_path(&ring, -2.0, OffsetStyle::default(), Tolerance::default()).unwrap();

    assert!(inside(&expanded, -1.0, 15.0));
    assert!(!inside(&expanded, 15.0, 15.0));
    assert!(inside(&expanded, 9.0, 15.0));

    assert!(!inside(&contracted, 1.0, 15.0));
    assert!(inside(&contracted, 3.0, 15.0));
    assert!(!inside(&contracted, 9.0, 15.0));
}

#[test]
fn evenodd_offset_handles_same_direction_hole() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(30.0, 0.0))
        .unwrap()
        .line_to(Point2::new(30.0, 30.0))
        .unwrap()
        .line_to(Point2::new(0.0, 30.0))
        .unwrap()
        .close()
        .unwrap();

    builder
        .move_to(Point2::new(10.0, 10.0))
        .unwrap()
        .line_to(Point2::new(20.0, 10.0))
        .unwrap()
        .line_to(Point2::new(20.0, 20.0))
        .unwrap()
        .line_to(Point2::new(10.0, 20.0))
        .unwrap()
        .close()
        .unwrap();

    let path = builder.finish().unwrap();
    let result = offset_path_with_fill_rule(
        &path,
        FillRule::EvenOdd,
        2.0,
        OffsetStyle::default(),
        Tolerance::default(),
    )
    .unwrap();

    assert!(inside(&result, -1.0, 15.0));
    assert!(inside(&result, 9.0, 15.0));
    assert!(!inside(&result, 15.0, 15.0));
}

#[test]
fn ellipse_outline_uses_adaptive_cubic_offsets() {
    let ellipse = ellipse_to_path(
        Ellipse::new(Point2::new(0.0, 0.0), 10.0, 5.0, Angle::radians(0.0)).unwrap(),
    )
    .unwrap();
    let style = StrokeStyle {
        width: 2.0,
        cap: StrokeCap::Butt,
        join: StrokeJoin::Round,
        miter_limit: 4.0,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };

    let result = outline_path(&ellipse, &style, Tolerance::default()).unwrap();
    let bounds = bounds_tuple(&result);

    assert!((bounds.0 + 11.0).abs() < 5.0e-3);
    assert!((bounds.1 + 6.0).abs() < 5.0e-3);
    assert!((bounds.2 - 11.0).abs() < 5.0e-3);
    assert!((bounds.3 - 6.0).abs() < 5.0e-3);
    assert!(
        result
            .subpaths()
            .iter()
            .flat_map(|subpath| subpath.segments())
            .any(|segment| matches!(segment, Segment::Cubic(_)))
    );
}

#[test]
fn large_coordinate_outline_remains_stable() {
    let offset = 1.0e9;
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(offset, offset))
        .unwrap()
        .line_to(Point2::new(offset + 100.0, offset))
        .unwrap();
    let path = builder.finish().unwrap();
    let style = StrokeStyle {
        width: 10.0,
        cap: StrokeCap::Round,
        join: StrokeJoin::Round,
        miter_limit: 4.0,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };

    let result = outline_path(&path, &style, Tolerance::default()).unwrap();

    assert!(inside(&result, offset + 50.0, offset + 4.0));
    assert!(!inside(&result, offset + 50.0, offset + 6.0));
}

#[test]
fn crossing_polyline_outline_is_cleaned_into_one_area() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(-10.0, -10.0))
        .unwrap()
        .line_to(Point2::new(10.0, 10.0))
        .unwrap()
        .line_to(Point2::new(-10.0, 10.0))
        .unwrap()
        .line_to(Point2::new(10.0, -10.0))
        .unwrap();
    let path = builder.finish().unwrap();
    let style = StrokeStyle {
        width: 4.0,
        cap: StrokeCap::Round,
        join: StrokeJoin::Round,
        miter_limit: 4.0,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };

    let result = outline_path(&path, &style, Tolerance::default()).unwrap();

    assert!(inside(&result, 0.0, 0.0));
    assert!(inside(&result, -9.0, -9.0));
    assert!(!inside(&result, 0.0, 6.0));
}

#[test]
fn inward_circle_offset_can_collapse_to_empty() {
    let circle = circle_to_path(Circle::new(Point2::new(0.0, 0.0), 5.0).unwrap()).unwrap();
    let result = offset_path(
        &circle,
        -6.0,
        OffsetStyle {
            join: StrokeJoin::Round,
            miter_limit: 4.0,
        },
        Tolerance::default(),
    )
    .unwrap();

    assert!(result.is_empty());
}
