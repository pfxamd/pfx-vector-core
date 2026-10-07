use pfx_vector_core::*;

fn line_path(start_x: f64, length: f64) -> Path {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(start_x, 0.0))
        .unwrap()
        .line_to(Point2::new(start_x + length, 0.0))
        .unwrap();
    builder.finish().unwrap()
}

fn only_subpath(path: &Path) -> &Subpath {
    assert_eq!(path.subpaths().len(), 1);
    &path.subpaths()[0]
}

#[test]
fn contour_length_counts_implicit_closing_edge() {
    let path = rect_to_path(Rect::new(0.0, 0.0, 10.0, 10.0).unwrap()).unwrap();
    let tolerance = Tolerance::default();

    assert!((path_length(&path, tolerance).unwrap() - 30.0).abs() < 1.0e-9);
    assert!((contour_length(only_subpath(&path), tolerance).unwrap() - 40.0).abs() < 1.0e-9);
}

#[test]
fn clamp_slice_of_open_line_uses_arc_length() {
    let path = line_path(0.0, 10.0);
    let tolerance = Tolerance::default();

    let sliced = slice_contour(
        only_subpath(&path),
        2.0,
        7.0,
        ContourSliceMode::Clamp,
        tolerance,
    )
    .unwrap();

    let subpath = only_subpath(&sliced);
    assert!(!subpath.is_closed());
    assert_eq!(subpath.segments().len(), 1);
    assert!(subpath.start().almost_eq(Point2::new(2.0, 0.0), tolerance));
    assert!(subpath.end().almost_eq(Point2::new(7.0, 0.0), tolerance));
    assert!((path_length(&sliced, tolerance).unwrap() - 5.0).abs() < 1.0e-8);
}

#[test]
fn clamp_slice_rejects_reversed_interval() {
    let path = line_path(0.0, 10.0);

    assert_eq!(
        slice_contour(
            only_subpath(&path),
            8.0,
            2.0,
            ContourSliceMode::Clamp,
            Tolerance::default(),
        ),
        Err(CoreError::InvalidGeometry)
    );
}

#[test]
fn slicing_preserves_native_quadratic_cubic_and_arc_segments() {
    let tolerance = Tolerance::default();

    let quadratic = QuadraticBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(20.0, 40.0),
        Point2::new(50.0, 0.0),
    );
    let mut q_builder = PathBuilder::new();
    q_builder
        .move_to(quadratic.p0)
        .unwrap()
        .quad_to(quadratic.p1, quadratic.p2)
        .unwrap();
    let q_path = q_builder.finish().unwrap();
    let q_total = contour_length(only_subpath(&q_path), tolerance).unwrap();
    let q_slice = slice_contour(
        only_subpath(&q_path),
        q_total * 0.2,
        q_total * 0.8,
        ContourSliceMode::Clamp,
        tolerance,
    )
    .unwrap();
    assert!(matches!(
        only_subpath(&q_slice).segments()[0],
        Segment::Quadratic(_)
    ));

    let cubic = CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(20.0, 60.0),
        Point2::new(80.0, -60.0),
        Point2::new(100.0, 0.0),
    );
    let mut c_builder = PathBuilder::new();
    c_builder
        .move_to(cubic.p0)
        .unwrap()
        .cubic_to(cubic.p1, cubic.p2, cubic.p3)
        .unwrap();
    let c_path = c_builder.finish().unwrap();
    let c_total = contour_length(only_subpath(&c_path), tolerance).unwrap();
    let c_slice = slice_contour(
        only_subpath(&c_path),
        c_total * 0.25,
        c_total * 0.75,
        ContourSliceMode::Clamp,
        tolerance,
    )
    .unwrap();
    assert!(matches!(
        only_subpath(&c_slice).segments()[0],
        Segment::Cubic(_)
    ));

    let arc = EllipticalArc::new(
        Point2::new(0.0, 0.0),
        30.0,
        20.0,
        Angle::radians(0.3),
        Angle::radians(0.0),
        Angle::radians(core::f64::consts::PI),
    );
    let mut a_builder = PathBuilder::new();
    a_builder
        .move_to(arc.point_at(0.0))
        .unwrap()
        .arc_to(arc)
        .unwrap();
    let a_path = a_builder.finish().unwrap();
    let a_total = contour_length(only_subpath(&a_path), tolerance).unwrap();
    let a_slice = slice_contour(
        only_subpath(&a_path),
        a_total * 0.1,
        a_total * 0.9,
        ContourSliceMode::Clamp,
        tolerance,
    )
    .unwrap();
    assert!(matches!(
        only_subpath(&a_slice).segments()[0],
        Segment::Arc(_)
    ));
}

#[test]
fn wrapped_closed_slice_crosses_implicit_close_seam() {
    let path = rect_to_path(Rect::new(0.0, 0.0, 10.0, 10.0).unwrap()).unwrap();
    let tolerance = Tolerance::default();

    let sliced = slice_contour(
        only_subpath(&path),
        35.0,
        5.0,
        ContourSliceMode::Wrap,
        tolerance,
    )
    .unwrap();

    let subpath = only_subpath(&sliced);
    assert!(!subpath.is_closed());
    assert!(subpath.start().almost_eq(Point2::new(0.0, 5.0), tolerance));
    assert!(subpath.end().almost_eq(Point2::new(5.0, 0.0), tolerance));
    assert!((path_length(&sliced, tolerance).unwrap() - 10.0).abs() < 1.0e-7);
}

#[test]
fn positive_wrapped_interval_may_cross_seam() {
    let path = rect_to_path(Rect::new(0.0, 0.0, 10.0, 10.0).unwrap()).unwrap();
    let tolerance = Tolerance::default();

    let sliced = slice_contour(
        only_subpath(&path),
        35.0,
        45.0,
        ContourSliceMode::Wrap,
        tolerance,
    )
    .unwrap();

    let subpath = only_subpath(&sliced);
    assert!(subpath.start().almost_eq(Point2::new(0.0, 5.0), tolerance));
    assert!(subpath.end().almost_eq(Point2::new(5.0, 0.0), tolerance));
}

#[test]
fn full_wrapped_cycle_preserves_closed_contour() {
    let path = rect_to_path(Rect::new(0.0, 0.0, 10.0, 10.0).unwrap()).unwrap();
    let tolerance = Tolerance::default();

    let sliced = slice_contour(
        only_subpath(&path),
        7.0,
        47.0,
        ContourSliceMode::Wrap,
        tolerance,
    )
    .unwrap();

    assert_eq!(sliced, path);
    assert!(only_subpath(&sliced).is_closed());
}

#[test]
fn wrap_mode_rejects_open_contours() {
    let path = line_path(0.0, 10.0);

    assert_eq!(
        slice_contour(
            only_subpath(&path),
            8.0,
            2.0,
            ContourSliceMode::Wrap,
            Tolerance::default(),
        ),
        Err(CoreError::UnsupportedCase)
    );
}

#[test]
fn split_contour_returns_complementary_open_pieces() {
    let path = rect_to_path(Rect::new(0.0, 0.0, 10.0, 10.0).unwrap()).unwrap();
    let tolerance = Tolerance::default();

    let (before, after) = split_contour_at_length(only_subpath(&path), 15.0, tolerance).unwrap();

    assert!(!only_subpath(&before).is_closed());
    assert!(!only_subpath(&after).is_closed());
    assert!((path_length(&before, tolerance).unwrap() - 15.0).abs() < 1.0e-7);
    assert!((path_length(&after, tolerance).unwrap() - 25.0).abs() < 1.0e-7);
    assert!(
        only_subpath(&before)
            .end()
            .almost_eq(only_subpath(&after).start(), tolerance)
    );
}

#[test]
fn zero_length_slice_is_empty() {
    let path = line_path(0.0, 10.0);
    let sliced = slice_contour(
        only_subpath(&path),
        4.0,
        4.0,
        ContourSliceMode::Clamp,
        Tolerance::default(),
    )
    .unwrap();

    assert!(sliced.is_empty());
}

#[test]
fn translated_contour_slice_remains_stable() {
    let path = line_path(1.0e9, 100.0);
    let tolerance = Tolerance::default();

    let sliced = slice_contour(
        only_subpath(&path),
        25.0,
        75.0,
        ContourSliceMode::Clamp,
        tolerance,
    )
    .unwrap();

    let subpath = only_subpath(&sliced);
    assert!((subpath.start().x - (1.0e9 + 25.0)).abs() < 1.0e-5);
    assert!((subpath.end().x - (1.0e9 + 75.0)).abs() < 1.0e-5);
}

#[test]
fn non_finite_slice_distance_is_rejected() {
    let path = line_path(0.0, 10.0);

    assert_eq!(
        slice_contour(
            only_subpath(&path),
            f64::NAN,
            5.0,
            ContourSliceMode::Clamp,
            Tolerance::default(),
        ),
        Err(CoreError::InvalidNumber)
    );
}
