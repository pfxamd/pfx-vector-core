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

fn mesh_area(mesh: &Mesh2D) -> f64 {
    mesh.indices
        .chunks_exact(3)
        .map(|triangle| {
            let a = mesh.vertices[triangle[0] as usize];
            let b = mesh.vertices[triangle[1] as usize];
            let c = mesh.vertices[triangle[2] as usize];
            (b - a).cross(c - a).abs() * 0.5
        })
        .sum()
}

#[test]
fn odd_dash_patterns_repeat_to_even_length() {
    let style = StrokeStyle {
        dash_array: vec![3.0, 2.0, 1.0],
        ..StrokeStyle::default()
    };

    assert_eq!(
        normalized_dash_pattern(&style),
        vec![3.0, 2.0, 1.0, 3.0, 2.0, 1.0]
    );
}

#[test]
fn line_dashes_split_at_expected_distances() {
    let path = line_path(0.0, 20.0);
    let style = StrokeStyle {
        dash_array: vec![5.0, 5.0],
        ..StrokeStyle::default()
    };

    let dashed = dash_path(&path, &style, Tolerance::default()).unwrap();
    assert_eq!(dashed.subpaths().len(), 2);
    assert_eq!(dashed.subpaths()[0].start(), Point2::new(0.0, 0.0));
    assert_eq!(dashed.subpaths()[0].end(), Point2::new(5.0, 0.0));
    assert_eq!(dashed.subpaths()[1].start(), Point2::new(10.0, 0.0));
    assert_eq!(dashed.subpaths()[1].end(), Point2::new(15.0, 0.0));
}

#[test]
fn dash_offset_changes_pattern_phase() {
    let path = line_path(0.0, 20.0);
    let style = StrokeStyle {
        dash_array: vec![5.0, 5.0],
        dash_offset: 2.0,
        ..StrokeStyle::default()
    };

    let tolerance = Tolerance::default();
    let dashed = dash_path(&path, &style, tolerance).unwrap();
    assert_eq!(dashed.subpaths().len(), 3);
    assert!(
        dashed.subpaths()[0]
            .end()
            .almost_eq(Point2::new(3.0, 0.0), tolerance)
    );
    assert!(
        dashed.subpaths()[1]
            .start()
            .almost_eq(Point2::new(8.0, 0.0), tolerance)
    );
    assert!(
        dashed.subpaths()[1]
            .end()
            .almost_eq(Point2::new(13.0, 0.0), tolerance)
    );
    assert!(
        dashed.subpaths()[2]
            .start()
            .almost_eq(Point2::new(18.0, 0.0), tolerance)
    );
}

#[test]
fn cubic_dashing_preserves_native_curve_segments() {
    let curve = CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(20.0, 40.0),
        Point2::new(80.0, -40.0),
        Point2::new(100.0, 0.0),
    );
    let mut builder = PathBuilder::new();
    builder
        .move_to(curve.p0)
        .unwrap()
        .cubic_to(curve.p1, curve.p2, curve.p3)
        .unwrap();
    let path = builder.finish().unwrap();
    let tolerance = Tolerance::default();
    let length = segment_length(Segment::Cubic(curve), tolerance).unwrap();
    let style = StrokeStyle {
        dash_array: vec![length * 0.5, length * 0.5],
        ..StrokeStyle::default()
    };

    let dashed = dash_path(&path, &style, tolerance).unwrap();
    assert_eq!(dashed.subpaths().len(), 1);
    assert!(matches!(
        dashed.subpaths()[0].segments()[0],
        Segment::Cubic(_)
    ));

    let expected_t =
        segment_parameter_at_length(Segment::Cubic(curve), length * 0.5, tolerance).unwrap();
    assert!(
        dashed.subpaths()[0]
            .end()
            .almost_eq(curve.point_at(expected_t), tolerance)
    );
}

#[test]
fn arc_dashing_preserves_native_arc_segments() {
    let arc = EllipticalArc::new(
        Point2::new(0.0, 0.0),
        30.0,
        20.0,
        Angle::radians(0.25),
        Angle::radians(0.0),
        Angle::radians(core::f64::consts::PI),
    );
    let mut builder = PathBuilder::new();
    builder
        .move_to(arc.point_at(0.0))
        .unwrap()
        .arc_to(arc)
        .unwrap();
    let path = builder.finish().unwrap();
    let tolerance = Tolerance::default();
    let length = segment_length(Segment::Arc(arc), tolerance).unwrap();
    let style = StrokeStyle {
        dash_array: vec![length * 0.4, length * 0.6],
        ..StrokeStyle::default()
    };

    let dashed = dash_path(&path, &style, tolerance).unwrap();
    assert_eq!(dashed.subpaths().len(), 1);
    assert!(matches!(
        dashed.subpaths()[0].segments()[0],
        Segment::Arc(_)
    ));
}

#[test]
fn closed_paths_merge_dashes_across_the_start_seam() {
    let path = rect_to_path(Rect::new(0.0, 0.0, 10.0, 10.0).unwrap()).unwrap();
    let style = StrokeStyle {
        dash_array: vec![8.0, 3.0],
        ..StrokeStyle::default()
    };

    let dashed = dash_path(&path, &style, Tolerance::default()).unwrap();
    assert_eq!(dashed.subpaths().len(), 3);
    assert!(dashed.subpaths().iter().all(|subpath| !subpath.is_closed()));
}

#[test]
fn dashed_hit_testing_excludes_gaps() {
    let path = line_path(0.0, 20.0);
    let style = StrokeStyle {
        width: 2.0,
        dash_array: vec![4.0, 4.0],
        ..StrokeStyle::default()
    };
    let tolerance = Tolerance::default();

    assert!(stroke_contains_point(&path, &style, Point2::new(2.0, 0.0), tolerance).unwrap());
    assert!(!stroke_contains_point(&path, &style, Point2::new(6.0, 0.0), tolerance).unwrap());

    let indexed = PathSpatialIndex::build(&path, tolerance).unwrap();
    assert!(
        indexed
            .stroke_contains_point(&style, Point2::new(2.0, 0.0))
            .unwrap()
    );
    assert!(
        !indexed
            .stroke_contains_point(&style, Point2::new(6.0, 0.0))
            .unwrap()
    );

    let incremental = IncrementalPathSpatialIndex::build(&path, tolerance).unwrap();
    assert!(
        incremental
            .stroke_contains_point(&style, Point2::new(2.0, 0.0))
            .unwrap()
    );
    assert!(
        !incremental
            .stroke_contains_point(&style, Point2::new(6.0, 0.0))
            .unwrap()
    );
}

#[test]
fn dashed_stroke_tessellation_matches_expected_line_area() {
    let path = line_path(0.0, 20.0);
    let style = StrokeStyle {
        width: 2.0,
        cap: StrokeCap::Butt,
        dash_array: vec![4.0, 4.0],
        ..StrokeStyle::default()
    };

    let mesh = tessellate_stroke(&path, &style, Tolerance::default()).unwrap();
    assert!((mesh_area(&mesh) - 24.0).abs() < 1.0e-7);
}

#[test]
fn translated_dashes_keep_the_same_local_geometry() {
    let tolerance = Tolerance::default();
    let style = StrokeStyle {
        dash_array: vec![5.0, 5.0],
        dash_offset: -2.0,
        ..StrokeStyle::default()
    };

    let local = dash_path(&line_path(0.0, 20.0), &style, tolerance).unwrap();
    let translated = dash_path(&line_path(1.0e9, 20.0), &style, tolerance).unwrap();

    assert_eq!(local.subpaths().len(), translated.subpaths().len());
    for (a, b) in local.subpaths().iter().zip(translated.subpaths()) {
        assert!((b.start().x - a.start().x - 1.0e9).abs() < 1.0e-6);
        assert!((b.end().x - a.end().x - 1.0e9).abs() < 1.0e-6);
    }
}

#[test]
fn invalid_dash_values_are_rejected() {
    let path = line_path(0.0, 10.0);
    let style = StrokeStyle {
        dash_array: vec![2.0, -1.0],
        ..StrokeStyle::default()
    };

    assert_eq!(
        dash_path(&path, &style, Tolerance::default()),
        Err(CoreError::InvalidGeometry)
    );
}
