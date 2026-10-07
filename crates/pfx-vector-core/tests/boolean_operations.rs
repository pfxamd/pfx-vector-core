use pfx_vector_core::*;

fn rect(x: f64, y: f64, width: f64, height: f64) -> Path {
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

#[test]
fn overlapping_rectangles_union() {
    let a = rect(0.0, 0.0, 10.0, 10.0);
    let b = rect(5.0, 0.0, 10.0, 10.0);

    let result = boolean_union(&a, &b, Tolerance::default()).unwrap();
    let bounds = path_bounds(&result);

    assert_eq!(result.subpaths().len(), 1);
    assert!(inside(&result, 2.0, 5.0));
    assert!(inside(&result, 12.0, 5.0));
    assert!(inside(&result, 7.5, 5.0));
    assert!(!inside(&result, 20.0, 5.0));
    assert!(matches!(
        bounds,
        Bounds::Finite { min, max }
            if min == Point2::new(0.0, 0.0) && max == Point2::new(15.0, 10.0)
    ));
}

#[test]
fn overlapping_rectangles_intersection() {
    let a = rect(0.0, 0.0, 10.0, 10.0);
    let b = rect(5.0, 0.0, 10.0, 10.0);

    let result = boolean_intersection(&a, &b, Tolerance::default()).unwrap();

    assert_eq!(result.subpaths().len(), 1);
    assert!(inside(&result, 7.5, 5.0));
    assert!(!inside(&result, 2.0, 5.0));
    assert!(!inside(&result, 12.0, 5.0));
}

#[test]
fn overlapping_rectangles_difference() {
    let a = rect(0.0, 0.0, 10.0, 10.0);
    let b = rect(5.0, 0.0, 10.0, 10.0);

    let result = boolean_difference(&a, &b, Tolerance::default()).unwrap();

    assert_eq!(result.subpaths().len(), 1);
    assert!(inside(&result, 2.0, 5.0));
    assert!(!inside(&result, 7.5, 5.0));
    assert!(!inside(&result, 12.0, 5.0));
}

#[test]
fn overlapping_rectangles_xor() {
    let a = rect(0.0, 0.0, 10.0, 10.0);
    let b = rect(5.0, 0.0, 10.0, 10.0);

    let result = boolean_xor(&a, &b, Tolerance::default()).unwrap();

    assert_eq!(result.subpaths().len(), 2);
    assert!(inside(&result, 2.0, 5.0));
    assert!(!inside(&result, 7.5, 5.0));
    assert!(inside(&result, 12.0, 5.0));
}

#[test]
fn difference_with_contained_rectangle_creates_hole() {
    let outer = rect(0.0, 0.0, 20.0, 20.0);
    let inner = rect(5.0, 5.0, 10.0, 10.0);

    let result = boolean_difference(&outer, &inner, Tolerance::default()).unwrap();

    assert_eq!(result.subpaths().len(), 2);
    assert!(inside(&result, 2.0, 2.0));
    assert!(!inside(&result, 10.0, 10.0));
}

#[test]
fn contained_shape_union_and_intersection_choose_expected_boundary() {
    let outer = rect(0.0, 0.0, 20.0, 20.0);
    let inner = rect(5.0, 5.0, 10.0, 10.0);

    let union = boolean_union(&outer, &inner, Tolerance::default()).unwrap();
    let intersection =
        boolean_intersection(&outer, &inner, Tolerance::default()).unwrap();

    assert_eq!(union.subpaths().len(), 1);
    assert!(inside(&union, 1.0, 1.0));
    assert!(inside(&union, 10.0, 10.0));

    assert_eq!(intersection.subpaths().len(), 1);
    assert!(!inside(&intersection, 1.0, 1.0));
    assert!(inside(&intersection, 10.0, 10.0));
}

#[test]
fn identical_paths_have_set_semantics() {
    let a = rect(0.0, 0.0, 10.0, 10.0);

    let union = boolean_union(&a, &a, Tolerance::default()).unwrap();
    let intersection = boolean_intersection(&a, &a, Tolerance::default()).unwrap();
    let difference = boolean_difference(&a, &a, Tolerance::default()).unwrap();
    let xor = boolean_xor(&a, &a, Tolerance::default()).unwrap();

    assert_eq!(union.subpaths().len(), 1);
    assert_eq!(intersection.subpaths().len(), 1);
    assert!(difference.is_empty());
    assert!(xor.is_empty());
}

#[test]
fn disjoint_paths_keep_separate_contours() {
    let a = rect(0.0, 0.0, 5.0, 5.0);
    let b = rect(10.0, 0.0, 5.0, 5.0);

    let union = boolean_union(&a, &b, Tolerance::default()).unwrap();
    let intersection = boolean_intersection(&a, &b, Tolerance::default()).unwrap();
    let difference = boolean_difference(&a, &b, Tolerance::default()).unwrap();
    let xor = boolean_xor(&a, &b, Tolerance::default()).unwrap();

    assert_eq!(union.subpaths().len(), 2);
    assert!(intersection.is_empty());
    assert_eq!(difference.subpaths().len(), 1);
    assert_eq!(xor.subpaths().len(), 2);
}

#[test]
fn boolean_preserves_curved_segments() {
    let left = circle_to_path(Circle::new(Point2::new(0.0, 0.0), 10.0).unwrap()).unwrap();
    let right = circle_to_path(Circle::new(Point2::new(10.0, 0.0), 10.0).unwrap()).unwrap();

    let result = boolean_intersection(&left, &right, Tolerance::default()).unwrap();

    assert_eq!(result.subpaths().len(), 1);
    assert!(result
        .subpaths()
        .iter()
        .flat_map(|subpath| subpath.segments())
        .all(|segment| matches!(segment, Segment::Arc(_))));
    assert!(inside(&result, 5.0, 0.0));
    assert!(!inside(&result, -8.0, 0.0));
    assert!(!inside(&result, 18.0, 0.0));
}

#[test]
fn open_paths_are_rejected() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap();
    let open = builder.finish().unwrap();
    let closed = rect(0.0, 0.0, 5.0, 5.0);

    assert_eq!(
        boolean_union(&open, &closed, Tolerance::default()),
        Err(CoreError::UnsupportedCase)
    );
}
