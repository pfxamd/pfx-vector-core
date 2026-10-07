use pfx_vector_core::*;

fn arc_path(arc: EllipticalArc) -> Path {
    let mut builder = PathBuilder::new();
    builder
        .move_to(arc.point_at(0.0))
        .unwrap()
        .arc_to(arc)
        .unwrap();
    builder.finish().unwrap()
}

fn transformed_arc(path: &Path) -> EllipticalArc {
    match path.subpaths()[0].segments()[0] {
        Segment::Arc(arc) => arc,
        other => panic!("expected transformed arc, got {other:?}"),
    }
}

fn assert_transform_matches_samples(arc: EllipticalArc, transform: Transform2D) {
    let tolerance = Tolerance::default();
    let source = arc_path(arc);
    let transformed = transform_path(&source, transform, tolerance).unwrap();
    let result = transformed_arc(&transformed);

    for parameter in [0.0, 0.125, 0.25, 0.5, 0.75, 0.875, 1.0] {
        let expected = transform.transform_point(arc.point_at(parameter));
        let actual = result.point_at(parameter);
        assert!(
            actual.almost_eq(expected, Tolerance::new(1.0e-8, 1.0e-10, 1.0e-10, 1.0e-4)),
            "parameter={parameter}, expected={expected:?}, actual={actual:?}"
        );
    }
}

#[test]
fn affine_transform_preserves_native_arc_geometry() {
    let arc = EllipticalArc::new(
        Point2::new(12.0, -8.0),
        40.0,
        17.0,
        Angle::degrees(23.0),
        Angle::degrees(-35.0),
        Angle::degrees(230.0),
    );
    let transform = Transform2D::translation(30.0, -12.0)
        .then(Transform2D::rotation(Angle::degrees(31.0)))
        .then(Transform2D::scale(1.8, 0.65))
        .then(Transform2D::skew_x(Angle::degrees(14.0)));

    assert_transform_matches_samples(arc, transform);
}

#[test]
fn reflection_preserves_arc_and_reverses_sweep_orientation() {
    let arc = EllipticalArc::new(
        Point2::new(-4.0, 7.0),
        18.0,
        9.0,
        Angle::degrees(-18.0),
        Angle::degrees(15.0),
        Angle::degrees(140.0),
    );
    let transform = Transform2D::translation(5.0, 2.0)
        .then(Transform2D::scale(-2.0, 1.25))
        .then(Transform2D::rotation(Angle::degrees(12.0)));

    let transformed =
        transform_elliptical_arc(arc, transform, Tolerance::default()).unwrap();

    assert!(transformed.sweep_angle.as_radians() < 0.0);
    assert_transform_matches_samples(arc, transform);
}

#[test]
fn full_ellipse_sweep_survives_affine_transform() {
    let arc = EllipticalArc::new(
        Point2::new(3.0, 5.0),
        22.0,
        11.0,
        Angle::degrees(37.0),
        Angle::degrees(70.0),
        Angle::radians(core::f64::consts::TAU),
    );
    let transform = Transform2D::scale(0.8, 1.7)
        .then(Transform2D::skew_y(Angle::degrees(-11.0)))
        .then(Transform2D::translation(-20.0, 9.0));

    assert_transform_matches_samples(arc, transform);
}

#[test]
fn singular_arc_transform_falls_back_to_line_geometry_in_paths() {
    let arc = EllipticalArc::new(
        Point2::new(0.0, 0.0),
        20.0,
        10.0,
        Angle::degrees(10.0),
        Angle::degrees(0.0),
        Angle::degrees(180.0),
    );
    let source = arc_path(arc);
    let singular = Transform2D::scale(1.0, 0.0);

    assert_eq!(
        transform_elliptical_arc(arc, singular, Tolerance::default()),
        Err(CoreError::SingularTransform)
    );

    let transformed = transform_path(&source, singular, Tolerance::default()).unwrap();
    assert!(
        transformed.subpaths()[0]
            .segments()
            .iter()
            .all(|segment| matches!(segment, Segment::Line(_)))
    );
    assert!(transformed.subpaths()[0].segments().len() > 1);
}

#[test]
fn non_finite_transform_is_rejected() {
    let arc = EllipticalArc::new(
        Point2::new(0.0, 0.0),
        4.0,
        2.0,
        Angle::degrees(0.0),
        Angle::degrees(0.0),
        Angle::degrees(90.0),
    );
    let transform = Transform2D::new(f64::NAN, 0.0, 0.0, 1.0, 0.0, 0.0);

    assert_eq!(
        transform_elliptical_arc(arc, transform, Tolerance::default()),
        Err(CoreError::InvalidNumber)
    );
}
