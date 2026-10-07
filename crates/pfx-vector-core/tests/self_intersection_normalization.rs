use pfx_vector_core::*;

fn bow_tie(offset: f64) -> Path {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(offset, offset))
        .unwrap()
        .line_to(Point2::new(offset + 10.0, offset + 10.0))
        .unwrap()
        .line_to(Point2::new(offset, offset + 10.0))
        .unwrap()
        .line_to(Point2::new(offset + 10.0, offset))
        .unwrap()
        .close()
        .unwrap();
    builder.finish().unwrap()
}

fn double_wound_square() -> Path {
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
        .line_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 10.0))
        .unwrap()
        .line_to(Point2::new(0.0, 10.0))
        .unwrap()
        .close()
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

fn inside(path: &Path, x: f64, y: f64) -> bool {
    classify_point(
        path,
        Point2::new(x, y),
        FillRule::NonZero,
        Tolerance::default(),
    )
    .unwrap()
        == PointClassification::Inside
}

#[test]
fn bow_tie_normalizes_to_two_simple_lobes() {
    let source = bow_tie(0.0);
    let normalized =
        normalize_self_intersections(&source, FillRule::EvenOdd, Tolerance::default()).unwrap();

    assert_eq!(normalized.subpaths().len(), 2);
    assert!(normalized.subpaths().iter().all(Subpath::is_closed));
    assert!(inside(&normalized, 5.0, 2.0));
    assert!(inside(&normalized, 5.0, 8.0));
    assert!(!inside(&normalized, 2.0, 5.0));
    assert!(!inside(&normalized, 8.0, 5.0));
}

#[test]
fn bow_tie_nonzero_and_evenodd_have_same_lobes() {
    let source = bow_tie(0.0);
    let nonzero =
        normalize_self_intersections(&source, FillRule::NonZero, Tolerance::default()).unwrap();
    let evenodd =
        normalize_self_intersections(&source, FillRule::EvenOdd, Tolerance::default()).unwrap();

    for x in 0..=10 {
        for y in 0..=10 {
            let point = Point2::new(f64::from(x) + 0.23, f64::from(y) + 0.37);
            let a =
                classify_point(&nonzero, point, FillRule::NonZero, Tolerance::default()).unwrap();
            let b =
                classify_point(&evenodd, point, FillRule::NonZero, Tolerance::default()).unwrap();
            assert_eq!(a, b, "point=({},{})", point.x, point.y);
        }
    }
}

#[test]
fn double_winding_respects_fill_rule() {
    let source = double_wound_square();

    let nonzero =
        normalize_self_intersections(&source, FillRule::NonZero, Tolerance::default()).unwrap();
    let evenodd =
        normalize_self_intersections(&source, FillRule::EvenOdd, Tolerance::default()).unwrap();

    assert_eq!(nonzero.subpaths().len(), 1);
    assert!(inside(&nonzero, 5.0, 5.0));
    assert!(evenodd.is_empty());
}

#[test]
fn normalization_is_idempotent() {
    let source = bow_tie(0.0);
    let first =
        normalize_self_intersections(&source, FillRule::EvenOdd, Tolerance::default()).unwrap();
    let second =
        normalize_self_intersections(&first, FillRule::NonZero, Tolerance::default()).unwrap();

    assert_eq!(first.segment_count(), second.segment_count());
    for x in 0..=10 {
        for y in 0..=10 {
            assert_eq!(
                inside(&first, f64::from(x) + 0.19, f64::from(y) + 0.31),
                inside(&second, f64::from(x) + 0.19, f64::from(y) + 0.31)
            );
        }
    }
}

#[test]
fn normalization_remains_stable_at_large_coordinates() {
    let offset = 1.0e9;
    let source = bow_tie(offset);
    let normalized =
        normalize_self_intersections(&source, FillRule::EvenOdd, Tolerance::default()).unwrap();

    assert_eq!(normalized.subpaths().len(), 2);
    assert!(inside(&normalized, offset + 5.0, offset + 2.0));
    assert!(inside(&normalized, offset + 5.0, offset + 8.0));
}

#[test]
fn tessellation_accepts_self_intersecting_fill_directly() {
    let source = bow_tie(0.0);
    let mesh = tessellate_fill(&source, FillRule::EvenOdd, Tolerance::default()).unwrap();

    assert!((mesh_area(&mesh) - 50.0).abs() < 1.0e-7);
    assert_eq!(mesh.triangle_count(), 2);
}

#[test]
fn tessellation_respects_double_winding_fill_rule() {
    let source = double_wound_square();

    let nonzero = tessellate_fill(&source, FillRule::NonZero, Tolerance::default()).unwrap();
    let evenodd = tessellate_fill(&source, FillRule::EvenOdd, Tolerance::default()).unwrap();

    assert!((mesh_area(&nonzero) - 100.0).abs() < 1.0e-7);
    assert!(evenodd.is_empty());
}

#[test]
fn boolean_operations_accept_self_intersecting_input() {
    let source = bow_tie(0.0);
    let cover = rect_to_path(Rect::new(0.0, 0.0, 10.0, 5.0).unwrap()).unwrap();

    let result = boolean_paths_with_fill_rules(
        &source,
        FillRule::EvenOdd,
        &cover,
        FillRule::NonZero,
        BooleanOperation::Intersection,
        Tolerance::default(),
    )
    .unwrap();

    assert!(inside(&result, 5.0, 2.0));
    assert!(!inside(&result, 5.0, 8.0));
}

#[test]
fn offset_accepts_self_intersecting_fill() {
    let source = bow_tie(0.0);
    let expanded = offset_path_with_fill_rule(
        &source,
        FillRule::EvenOdd,
        1.0,
        OffsetStyle::default(),
        Tolerance::default(),
    )
    .unwrap();

    assert!(!expanded.is_empty());
    assert!(inside(&expanded, 5.0, 2.0));
    assert!(inside(&expanded, 5.0, 8.0));
}

#[test]
fn nested_same_direction_contour_is_normalized_by_fill_semantics() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(20.0, 0.0))
        .unwrap()
        .line_to(Point2::new(20.0, 20.0))
        .unwrap()
        .line_to(Point2::new(0.0, 20.0))
        .unwrap()
        .close()
        .unwrap()
        .move_to(Point2::new(5.0, 5.0))
        .unwrap()
        .line_to(Point2::new(15.0, 5.0))
        .unwrap()
        .line_to(Point2::new(15.0, 15.0))
        .unwrap()
        .line_to(Point2::new(5.0, 15.0))
        .unwrap()
        .close()
        .unwrap();
    let source = builder.finish().unwrap();

    let nonzero =
        normalize_self_intersections(&source, FillRule::NonZero, Tolerance::default()).unwrap();
    let evenodd =
        normalize_self_intersections(&source, FillRule::EvenOdd, Tolerance::default()).unwrap();

    assert_eq!(nonzero.subpaths().len(), 1);
    assert_eq!(evenodd.subpaths().len(), 2);
    assert!(inside(&nonzero, 10.0, 10.0));
    assert!(!inside(&evenodd, 10.0, 10.0));
}


#[test]
fn intrinsic_cubic_self_intersection_is_split_without_flattening() {
    let curve = CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(-20.0, -15.0),
        Point2::new(-10.0, -10.0),
        Point2::new(10.0, 10.0),
    );
    let first = 0.05358983848622473;
    let second = 0.7464101615137753;
    assert!(curve.point_at(first).distance_to(curve.point_at(second)) < 1.0e-9);

    let mut builder = PathBuilder::new();
    builder
        .move_to(curve.p0)
        .unwrap()
        .cubic_to(curve.p1, curve.p2, curve.p3)
        .unwrap()
        .close()
        .unwrap();
    let source = builder.finish().unwrap();

    let normalized =
        normalize_self_intersections(&source, FillRule::NonZero, Tolerance::default()).unwrap();

    assert!(normalized.segment_count() > 2);
    assert!(normalized.subpaths().iter().all(Subpath::is_closed));
    assert!(normalized
        .subpaths()
        .iter()
        .flat_map(|subpath| subpath.segments())
        .any(|segment| matches!(segment, Segment::Cubic(_))));
}

#[test]
fn tessellation_accepts_intrinsically_self_intersecting_cubic() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(-20.0, -15.0),
            Point2::new(-10.0, -10.0),
            Point2::new(10.0, 10.0),
        )
        .unwrap()
        .close()
        .unwrap();
    let source = builder.finish().unwrap();

    let mesh = tessellate_fill(&source, FillRule::NonZero, Tolerance::default()).unwrap();

    assert!(!mesh.is_empty());
    assert!(mesh.triangle_count() >= 2);
}

#[test]
fn ordinary_non_looping_cubic_is_not_artificially_split() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(3.0, 8.0),
            Point2::new(7.0, 8.0),
            Point2::new(10.0, 0.0),
        )
        .unwrap()
        .close()
        .unwrap();
    let source = builder.finish().unwrap();
    let normalized =
        normalize_self_intersections(&source, FillRule::NonZero, Tolerance::default()).unwrap();

    assert_eq!(normalized.subpaths().len(), 1);
    assert_eq!(normalized.segment_count(), 2);
}
