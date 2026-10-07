use pfx_vector_core::*;

fn line_path() -> Path {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap();
    builder.finish().unwrap()
}

fn corner_path() -> Path {
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

#[test]
fn cap_modes_have_distinct_hit_regions() {
    let path = line_path();
    let tolerance = Tolerance::default();
    let butt = StrokeStyle {
        width: 4.0,
        cap: StrokeCap::Butt,
        ..StrokeStyle::default()
    };
    let round = StrokeStyle {
        cap: StrokeCap::Round,
        ..butt.clone()
    };
    let square = StrokeStyle {
        cap: StrokeCap::Square,
        ..butt.clone()
    };

    assert!(!stroke_contains_point(&path, &butt, Point2::new(-1.5, 0.0), tolerance).unwrap());
    assert!(stroke_contains_point(&path, &round, Point2::new(-1.5, 0.0), tolerance).unwrap());
    assert!(!stroke_contains_point(&path, &round, Point2::new(-1.5, 1.5), tolerance).unwrap());
    assert!(stroke_contains_point(&path, &square, Point2::new(-1.5, 1.5), tolerance).unwrap());
}

#[test]
fn join_modes_have_distinct_hit_regions() {
    let path = corner_path();
    let tolerance = Tolerance::default();
    let miter = StrokeStyle {
        width: 4.0,
        join: StrokeJoin::Miter,
        miter_limit: 4.0,
        ..StrokeStyle::default()
    };
    let bevel = StrokeStyle {
        join: StrokeJoin::Bevel,
        ..miter.clone()
    };
    let round = StrokeStyle {
        join: StrokeJoin::Round,
        ..miter.clone()
    };

    let miter_tip = Point2::new(11.75, -1.75);
    assert!(stroke_contains_point(&path, &miter, miter_tip, tolerance).unwrap());
    assert!(!stroke_contains_point(&path, &bevel, miter_tip, tolerance).unwrap());

    let round_only = Point2::new(11.4, -1.4);
    assert!(stroke_contains_point(&path, &round, round_only, tolerance).unwrap());
    assert!(!stroke_contains_point(&path, &bevel, round_only, tolerance).unwrap());
}

#[test]
fn dashed_round_caps_and_gaps_are_respected() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(20.0, 0.0))
        .unwrap();
    let path = builder.finish().unwrap();
    let style = StrokeStyle {
        width: 2.0,
        cap: StrokeCap::Round,
        dash_array: vec![4.0, 4.0],
        ..StrokeStyle::default()
    };
    let tolerance = Tolerance::default();

    assert!(stroke_contains_point(&path, &style, Point2::new(4.75, 0.0), tolerance).unwrap());
    assert!(!stroke_contains_point(&path, &style, Point2::new(6.0, 0.0), tolerance).unwrap());
}

#[test]
fn cached_and_spatial_indexes_match_direct_stroke_hits() {
    let path = corner_path();
    let style = StrokeStyle {
        width: 4.0,
        cap: StrokeCap::Square,
        join: StrokeJoin::Miter,
        miter_limit: 4.0,
        ..StrokeStyle::default()
    };
    let tolerance = Tolerance::default();
    let cached = StrokeHitIndex::build(&path, &style, tolerance).unwrap();
    let spatial = PathSpatialIndex::build(&path, tolerance).unwrap();
    let incremental = IncrementalPathSpatialIndex::build(&path, tolerance).unwrap();

    for point in [
        Point2::new(-1.5, 1.5),
        Point2::new(11.75, -1.75),
        Point2::new(8.0, 2.0),
        Point2::new(20.0, 20.0),
    ] {
        let direct = stroke_contains_point(&path, &style, point, tolerance).unwrap();
        assert_eq!(cached.contains_point(point).unwrap(), direct);
        assert_eq!(spatial.stroke_contains_point(&style, point).unwrap(), direct);
        assert_eq!(incremental.stroke_contains_point(&style, point).unwrap(), direct);
    }
}

#[test]
fn precise_stroke_bounds_include_cap_and_miter_extent() {
    let path = corner_path();
    let style = StrokeStyle {
        width: 4.0,
        cap: StrokeCap::Square,
        join: StrokeJoin::Miter,
        miter_limit: 4.0,
        ..StrokeStyle::default()
    };

    let Bounds::Finite { min, max } =
        stroke_bounds(&path, &style, Tolerance::default()).unwrap()
    else {
        panic!("stroke bounds must be finite");
    };

    assert!(min.x <= -2.0);
    assert!(min.y <= -2.0);
    assert!(max.x >= 12.0);
    assert!(max.y >= 12.0);
}

#[test]
fn large_coordinate_round_cap_hit_is_stable() {
    let base = 1.0e9;
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(base, base))
        .unwrap()
        .line_to(Point2::new(base + 100.0, base))
        .unwrap();
    let path = builder.finish().unwrap();
    let style = StrokeStyle {
        width: 4.0,
        cap: StrokeCap::Round,
        ..StrokeStyle::default()
    };
    let point = Point2::new(base - 1.5, base);
    let tolerance = Tolerance::default();

    assert!(stroke_contains_point(&path, &style, point, tolerance).unwrap());
    assert!(
        PathSpatialIndex::build(&path, tolerance)
            .unwrap()
            .stroke_contains_point(&style, point)
            .unwrap()
    );
}

#[test]
fn overflowing_stroke_reach_is_rejected() {
    let path = line_path();
    let style = StrokeStyle {
        width: f64::MAX,
        miter_limit: f64::MAX,
        ..StrokeStyle::default()
    };

    assert_eq!(
        stroke_contains_point(
            &path,
            &style,
            Point2::new(0.0, 0.0),
            Tolerance::default(),
        ),
        Err(CoreError::InvalidGeometry)
    );
}
