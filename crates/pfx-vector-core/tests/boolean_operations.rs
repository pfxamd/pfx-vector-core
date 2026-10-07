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
    let intersection = boolean_intersection(&outer, &inner, Tolerance::default()).unwrap();

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
    assert!(
        result
            .subpaths()
            .iter()
            .flat_map(|subpath| subpath.segments())
            .all(|segment| matches!(segment, Segment::Arc(_)))
    );
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


#[test]
fn shared_edge_is_removed_from_union_boundary() {
    let a = rect(0.0, 0.0, 10.0, 10.0);
    let b = rect(10.0, 0.0, 10.0, 10.0);

    let union = boolean_union(&a, &b, Tolerance::default()).unwrap();
    let intersection = boolean_intersection(&a, &b, Tolerance::default()).unwrap();
    let xor = boolean_xor(&a, &b, Tolerance::default()).unwrap();

    assert_eq!(union.subpaths().len(), 1);
    assert!(inside(&union, 5.0, 5.0));
    assert!(inside(&union, 15.0, 5.0));
    assert!(intersection.is_empty());
    assert_eq!(xor.subpaths().len(), 1);
    assert!(inside(&xor, 5.0, 5.0));
    assert!(inside(&xor, 15.0, 5.0));
}

#[test]
fn boolean_commutative_operations_match_by_membership() {
    let a = circle_to_path(Circle::new(Point2::new(0.0, 0.0), 10.0).unwrap()).unwrap();
    let b = circle_to_path(Circle::new(Point2::new(8.0, 2.0), 8.0).unwrap()).unwrap();

    for operation in [
        BooleanOperation::Union,
        BooleanOperation::Intersection,
        BooleanOperation::Xor,
    ] {
        let forward = boolean_paths(&a, &b, operation, Tolerance::default()).unwrap();
        let reverse = boolean_paths(&b, &a, operation, Tolerance::default()).unwrap();

        for x in -12..=16 {
            for y in -12..=12 {
                let x = f64::from(x);
                let y = f64::from(y);
                assert_eq!(
                    inside(&forward, x, y),
                    inside(&reverse, x, y),
                    "operation={operation:?}, point=({x},{y})"
                );
            }
        }
    }
}

#[test]
fn reversing_input_orientation_does_not_change_set_result() {
    let a = rect(0.0, 0.0, 10.0, 10.0);
    let b = rect(5.0, -5.0, 10.0, 20.0);
    let reversed_b = b.reversed();

    for operation in [
        BooleanOperation::Union,
        BooleanOperation::Intersection,
        BooleanOperation::Difference,
        BooleanOperation::Xor,
    ] {
        let normal = boolean_paths(&a, &b, operation, Tolerance::default()).unwrap();
        let reversed =
            boolean_paths(&a, &reversed_b, operation, Tolerance::default()).unwrap();

        for x in -2..=17 {
            for y in -7..=17 {
                let x = f64::from(x);
                let y = f64::from(y);
                assert_eq!(
                    inside(&normal, x, y),
                    inside(&reversed, x, y),
                    "operation={operation:?}, point=({x},{y})"
                );
            }
        }
    }
}

#[test]
fn large_translated_coordinates_remain_stable() {
    let offset = 1.0e9;
    let a = rect(offset, offset, 10.0, 10.0);
    let b = rect(offset + 5.0, offset, 10.0, 10.0);

    let union = boolean_union(&a, &b, Tolerance::default()).unwrap();
    let intersection = boolean_intersection(&a, &b, Tolerance::default()).unwrap();

    assert!(inside(&union, offset + 2.0, offset + 5.0));
    assert!(inside(&union, offset + 12.0, offset + 5.0));
    assert!(inside(&intersection, offset + 7.0, offset + 5.0));
    assert!(!inside(&intersection, offset + 2.0, offset + 5.0));
}

#[test]
fn evenodd_input_hole_is_preserved_by_boolean_topology() {
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
        .unwrap();

    builder
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

    let ring = builder.finish().unwrap();
    let cover = rect(-5.0, -5.0, 30.0, 30.0);

    let result = boolean_paths_with_fill_rules(
        &ring,
        FillRule::EvenOdd,
        &cover,
        FillRule::NonZero,
        BooleanOperation::Intersection,
        Tolerance::default(),
    )
    .unwrap();

    assert_eq!(result.subpaths().len(), 2);
    assert!(inside(&result, 2.0, 2.0));
    assert!(!inside(&result, 10.0, 10.0));
}

#[test]
fn tangent_circles_do_not_create_intersection_area() {
    let left = circle_to_path(Circle::new(Point2::new(0.0, 0.0), 10.0).unwrap()).unwrap();
    let right = circle_to_path(Circle::new(Point2::new(20.0, 0.0), 10.0).unwrap()).unwrap();

    let intersection =
        boolean_intersection(&left, &right, Tolerance::default()).unwrap();
    let union = boolean_union(&left, &right, Tolerance::default()).unwrap();

    assert!(intersection.is_empty());
    assert!(inside(&union, -5.0, 0.0));
    assert!(inside(&union, 25.0, 0.0));
    assert!(!inside(&union, 10.0, 5.0));
}
