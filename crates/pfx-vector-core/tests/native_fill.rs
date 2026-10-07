use pfx_vector_core::*;

fn closed_cubic_cap() -> Path {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(0.0, 10.0),
            Point2::new(10.0, 10.0),
            Point2::new(10.0, 0.0),
        )
        .unwrap()
        .close()
        .unwrap();
    builder.finish().unwrap()
}

#[test]
fn cubic_fill_is_independent_of_flatness() {
    let path = closed_cubic_cap();
    let coarse = Tolerance {
        flatness: 1000.0,
        ..Tolerance::default()
    };

    assert_eq!(
        classify_point(&path, Point2::new(5.0, 4.0), FillRule::NonZero, coarse).unwrap(),
        PointClassification::Inside
    );
    assert_eq!(
        classify_point(&path, Point2::new(5.0, 8.0), FillRule::NonZero, coarse).unwrap(),
        PointClassification::Outside
    );
}

#[test]
fn quadratic_fill_is_independent_of_flatness() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .quad_to(Point2::new(5.0, 10.0), Point2::new(10.0, 0.0))
        .unwrap()
        .close()
        .unwrap();
    let path = builder.finish().unwrap();
    let coarse = Tolerance {
        flatness: 1000.0,
        ..Tolerance::default()
    };

    assert!(contains_point(&path, Point2::new(5.0, 2.0), FillRule::NonZero, coarse).unwrap());
    assert!(!contains_point(&path, Point2::new(5.0, 6.0), FillRule::NonZero, coarse).unwrap());
}

#[test]
fn elliptical_arc_fill_is_independent_of_flatness() {
    let ellipse = ellipse_to_path(
        Ellipse::new(Point2::new(0.0, 0.0), 12.0, 5.0, Angle::degrees(27.0)).unwrap(),
    )
    .unwrap();
    let coarse = Tolerance {
        flatness: 1000.0,
        ..Tolerance::default()
    };

    assert_eq!(
        classify_point(&ellipse, Point2::new(0.0, 0.0), FillRule::NonZero, coarse).unwrap(),
        PointClassification::Inside
    );
    assert_eq!(
        classify_point(&ellipse, Point2::new(20.0, 0.0), FillRule::NonZero, coarse).unwrap(),
        PointClassification::Outside
    );
}

#[test]
fn native_boundary_detection_uses_source_curve() {
    let path = closed_cubic_cap();
    let coarse = Tolerance {
        flatness: 1000.0,
        ..Tolerance::default()
    };
    let boundary = path.subpaths()[0].segments()[0].point_at(0.5);

    assert_eq!(
        classify_point(&path, boundary, FillRule::NonZero, coarse).unwrap(),
        PointClassification::Boundary
    );
}

#[test]
fn horizontal_tangent_does_not_change_winding() {
    let circle = circle_to_path(Circle::new(Point2::new(0.0, 0.0), 10.0).unwrap()).unwrap();
    let tolerance = Tolerance::default();

    assert_eq!(
        classify_point(
            &circle,
            Point2::new(-20.0, 10.0),
            FillRule::NonZero,
            tolerance
        )
        .unwrap(),
        PointClassification::Outside
    );
    assert_eq!(
        classify_point(
            &circle,
            Point2::new(0.0, 10.0),
            FillRule::NonZero,
            tolerance
        )
        .unwrap(),
        PointClassification::Boundary
    );
}

#[test]
fn evenodd_and_nonzero_preserve_double_winding_semantics() {
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
    let path = builder.finish().unwrap();
    let point = Point2::new(5.0, 5.0);

    assert_eq!(
        classify_point(&path, point, FillRule::NonZero, Tolerance::default()).unwrap(),
        PointClassification::Inside
    );
    assert_eq!(
        classify_point(&path, point, FillRule::EvenOdd, Tolerance::default()).unwrap(),
        PointClassification::Outside
    );
}

#[test]
fn native_fill_remains_stable_at_large_coordinates() {
    let base = 1.0e12;
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(base, base))
        .unwrap()
        .cubic_to(
            Point2::new(base, base + 100.0),
            Point2::new(base + 100.0, base + 100.0),
            Point2::new(base + 100.0, base),
        )
        .unwrap()
        .close()
        .unwrap();
    let path = builder.finish().unwrap();
    let tolerance = Tolerance {
        flatness: 1000.0,
        ..Tolerance::default()
    };

    assert_eq!(
        classify_point(
            &path,
            Point2::new(base + 50.0, base + 30.0),
            FillRule::NonZero,
            tolerance
        )
        .unwrap(),
        PointClassification::Inside
    );
}

#[test]
fn non_finite_fill_query_is_rejected() {
    let path = closed_cubic_cap();

    assert_eq!(
        classify_point(
            &path,
            Point2::new(f64::NAN, 0.0),
            FillRule::NonZero,
            Tolerance::default()
        ),
        Err(CoreError::InvalidNumber)
    );
}

#[test]
fn full_circle_near_endpoint_ray_keeps_correct_winding() {
    let circle = circle_to_path(Circle::new(Point2::new(0.0, 0.0), 10.0).unwrap()).unwrap();
    let tolerance = Tolerance::default();
    let ray_y = -1.0e-15;

    assert_eq!(
        classify_point(
            &circle,
            Point2::new(-9.9996, ray_y),
            FillRule::NonZero,
            tolerance
        )
        .unwrap(),
        PointClassification::Inside
    );
    assert_eq!(
        classify_point(
            &circle,
            Point2::new(-10.0004, ray_y),
            FillRule::NonZero,
            tolerance
        )
        .unwrap(),
        PointClassification::Outside
    );
}
