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

fn exact_offset_point(segment: Segment, t: f64, distance: f64) -> Point2 {
    let derivative = match segment {
        Segment::Line(line) => line.direction(),
        Segment::Quadratic(curve) => curve.derivative_at(t),
        Segment::Cubic(curve) => curve.derivative_at(t),
        Segment::Arc(arc) => arc.derivative_at(t),
    };
    let tangent = derivative.normalized(Tolerance::default()).unwrap();
    segment.point_at(t) + tangent.perpendicular() * distance
}

#[test]
fn cubic_offset_stays_within_requested_flatness() {
    let curve = CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(20.0, 80.0),
        Point2::new(80.0, -70.0),
        Point2::new(120.0, 10.0),
    );
    let mut builder = PathBuilder::new();
    builder
        .move_to(curve.p0)
        .unwrap()
        .cubic_to(curve.p1, curve.p2, curve.p3)
        .unwrap();
    let path = builder.finish().unwrap();
    let tolerance = Tolerance {
        flatness: 5.0e-4,
        ..Tolerance::default()
    };
    let distance = 7.0;
    let offset = offset_path(&path, distance, OffsetStyle::default(), tolerance).unwrap();

    for step in 0..=100 {
        let t = step as f64 / 100.0;
        let expected = exact_offset_point(Segment::Cubic(curve), t, distance);
        let nearest = closest_point(&offset, expected, tolerance).unwrap();
        assert!(
            nearest.distance <= tolerance.flatness * 1.5,
            "t={t}, error={}, flatness={}",
            nearest.distance,
            tolerance.flatness
        );
    }
}

#[test]
fn elliptical_arc_offset_stays_within_requested_flatness() {
    let arc = EllipticalArc::new(
        Point2::new(15.0, -8.0),
        36.0,
        11.0,
        Angle::degrees(27.0),
        Angle::degrees(-35.0),
        Angle::degrees(250.0),
    );
    let mut builder = PathBuilder::new();
    builder
        .move_to(arc.point_at(0.0))
        .unwrap()
        .arc_to(arc)
        .unwrap();
    let path = builder.finish().unwrap();
    let tolerance = Tolerance {
        flatness: 5.0e-4,
        ..Tolerance::default()
    };
    let distance = 2.0;
    let offset = offset_path(&path, distance, OffsetStyle::default(), tolerance).unwrap();

    for step in 0..=120 {
        let t = step as f64 / 120.0;
        let expected = exact_offset_point(Segment::Arc(arc), t, distance);
        let nearest = closest_point(&offset, expected, tolerance).unwrap();
        assert!(
            nearest.distance <= tolerance.flatness * 1.5,
            "t={t}, error={}, flatness={}",
            nearest.distance,
            tolerance.flatness
        );
    }
}

#[test]
fn high_curvature_offset_splits_before_curvature_singularity() {
    let curve = QuadraticBezier::new(
        Point2::new(-10.0, 0.0),
        Point2::new(0.0, 20.0),
        Point2::new(10.0, 0.0),
    );
    let mut builder = PathBuilder::new();
    builder
        .move_to(curve.p0)
        .unwrap()
        .quad_to(curve.p1, curve.p2)
        .unwrap();
    let path = builder.finish().unwrap();
    let tolerance = Tolerance {
        flatness: 1.0e-3,
        ..Tolerance::default()
    };

    let result = offset_path(&path, 4.0, OffsetStyle::default(), tolerance).unwrap();
    let segments = only_subpath(&result).segments();

    assert!(segments.len() > 1);
    assert!(
        segments
            .iter()
            .all(|segment| matches!(segment, Segment::Cubic(_)))
    );
    assert!(
        segments
            .iter()
            .all(|segment| { segment.start().is_finite() && segment.end().is_finite() })
    );
}

#[test]
fn large_coordinate_cubic_offset_preserves_precision() {
    let base = 1.0e9;
    let curve = CubicBezier::new(
        Point2::new(base, base),
        Point2::new(base + 30.0, base + 70.0),
        Point2::new(base + 90.0, base - 60.0),
        Point2::new(base + 140.0, base + 15.0),
    );
    let mut builder = PathBuilder::new();
    builder
        .move_to(curve.p0)
        .unwrap()
        .cubic_to(curve.p1, curve.p2, curve.p3)
        .unwrap();
    let path = builder.finish().unwrap();
    let tolerance = Tolerance {
        flatness: 1.0e-3,
        ..Tolerance::default()
    };
    let distance = 6.0;
    let offset = offset_path(&path, distance, OffsetStyle::default(), tolerance).unwrap();

    for step in 0..=80 {
        let t = step as f64 / 80.0;
        let expected = exact_offset_point(Segment::Cubic(curve), t, distance);
        let nearest = closest_point(&offset, expected, tolerance).unwrap();
        assert!(
            nearest.distance <= 2.0e-3,
            "t={t}, error={}",
            nearest.distance
        );
    }
}
