use pfx_vector_core::*;

fn path_from_points(points: &[Point2], closed: bool) -> Path {
    let mut builder = PathBuilder::new();
    builder.move_to(points[0]).unwrap();
    for &point in &points[1..] {
        builder.line_to(point).unwrap();
    }
    if closed {
        builder.close().unwrap();
    }
    builder.finish().unwrap()
}

fn max_sample_distance(path: &Path, points: &[Point2], tolerance: Tolerance) -> f64 {
    points
        .iter()
        .map(|&point| closest_point(path, point, tolerance).unwrap().distance)
        .fold(0.0, f64::max)
}

#[test]
fn cleanup_removes_duplicate_and_collinear_line_noise() {
    let points = [
        Point2::new(0.0, 0.0),
        Point2::new(1.0e-10, 0.0),
        Point2::new(3.0, 1.0e-9),
        Point2::new(6.0, -1.0e-9),
        Point2::new(10.0, 0.0),
    ];
    let path = path_from_points(&points, false);
    let cleaned = cleanup_path(
        &path,
        CleanupOptions {
            point_tolerance: 1.0e-8,
            collinear_tolerance: 1.0e-6,
        },
        Tolerance::default(),
    )
    .unwrap();

    assert_eq!(cleaned.segment_count(), 1);
    let segment = cleaned.subpaths()[0].segments()[0];
    assert!(matches!(segment, Segment::Line(_)));
    assert_eq!(segment.start(), Point2::new(0.0, 0.0));
    assert_eq!(segment.end(), Point2::new(10.0, 0.0));
}

#[test]
fn cleanup_converts_nearly_linear_bezier_to_line() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(3.0, 1.0e-8),
            Point2::new(7.0, -1.0e-8),
            Point2::new(10.0, 0.0),
        )
        .unwrap();
    let path = builder.finish().unwrap();

    let cleaned = cleanup_path(
        &path,
        CleanupOptions {
            point_tolerance: 1.0e-9,
            collinear_tolerance: 1.0e-6,
        },
        Tolerance::default(),
    )
    .unwrap();

    assert!(matches!(
        cleaned.subpaths()[0].segments()[0],
        Segment::Line(_)
    ));
}

#[test]
fn cleanup_preserves_real_curve_geometry() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(0.0, 10.0),
            Point2::new(10.0, 10.0),
            Point2::new(10.0, 0.0),
        )
        .unwrap();
    let path = builder.finish().unwrap();
    let cleaned = cleanup_path(&path, CleanupOptions::default(), Tolerance::default()).unwrap();

    assert!(matches!(
        cleaned.subpaths()[0].segments()[0],
        Segment::Cubic(_)
    ));
}

#[test]
fn simplify_dense_straight_polyline_to_single_segment() {
    let points: Vec<_> = (0..=100)
        .map(|index| Point2::new(index as f64, (index as f64 * 0.17).sin() * 1.0e-3))
        .collect();
    let path = path_from_points(&points, false);
    let simplified = simplify_path(&path, 0.01, Tolerance::default()).unwrap();

    assert_eq!(simplified.segment_count(), 1);
}

#[test]
fn simplify_preserves_closed_square_corners() {
    let points = [
        Point2::new(0.0, 0.0),
        Point2::new(5.0, 0.0),
        Point2::new(10.0, 0.0),
        Point2::new(10.0, 5.0),
        Point2::new(10.0, 10.0),
        Point2::new(5.0, 10.0),
        Point2::new(0.0, 10.0),
        Point2::new(0.0, 5.0),
    ];
    let path = path_from_points(&points, true);
    let simplified = simplify_path(&path, 0.1, Tolerance::default()).unwrap();

    assert_eq!(simplified.subpaths().len(), 1);
    assert!(simplified.subpaths()[0].is_closed());
    assert_eq!(simplified.segment_count(), 4);
}

#[test]
fn larger_simplification_tolerance_never_adds_segments() {
    let points: Vec<_> = (0..=80)
        .map(|index| {
            let x = index as f64 * 0.25;
            Point2::new(x, (x * 0.8).sin())
        })
        .collect();
    let path = path_from_points(&points, false);
    let fine = simplify_path(&path, 0.02, Tolerance::default()).unwrap();
    let coarse = simplify_path(&path, 0.2, Tolerance::default()).unwrap();

    assert!(coarse.segment_count() <= fine.segment_count());
}

#[test]
fn simplification_is_translation_stable() {
    let base: Vec<_> = (0..=50)
        .map(|index| {
            let x = index as f64;
            Point2::new(x, (x * 0.2).sin())
        })
        .collect();
    let translated: Vec<_> = base
        .iter()
        .map(|point| Point2::new(point.x + 1.0e9, point.y - 1.0e9))
        .collect();

    let a = simplify_path(&path_from_points(&base, false), 0.05, Tolerance::default()).unwrap();
    let b = simplify_path(
        &path_from_points(&translated, false),
        0.05,
        Tolerance::default(),
    )
    .unwrap();

    assert_eq!(a.segment_count(), b.segment_count());
}

#[test]
fn straight_polyline_fits_to_one_cubic() {
    let points: Vec<_> = (0..=20)
        .map(|index| Point2::new(index as f64, 0.0))
        .collect();
    let curves = fit_polyline_cubics(&points, false, 1.0e-6, Tolerance::default()).unwrap();

    assert_eq!(curves.len(), 1);
    assert_eq!(curves[0].p0, points[0]);
    assert_eq!(curves[0].p3, points[20]);
}

#[test]
fn sampled_cubic_refits_within_requested_error() {
    let source = CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(20.0, 40.0),
        Point2::new(80.0, -40.0),
        Point2::new(100.0, 0.0),
    );
    let samples: Vec<_> = (0..=80)
        .map(|index| source.point_at(index as f64 / 80.0))
        .collect();
    let curves = fit_polyline_cubics(&samples, false, 0.05, Tolerance::default()).unwrap();

    let mut builder = PathBuilder::new();
    builder.move_to(curves[0].p0).unwrap();
    for curve in curves {
        builder.cubic_to(curve.p1, curve.p2, curve.p3).unwrap();
    }
    let fitted = builder.finish().unwrap();

    assert!(max_sample_distance(&fitted, &samples, Tolerance::default()) <= 0.06);
    assert!(fitted.segment_count() < samples.len() / 4);
}

#[test]
fn closed_circle_samples_fit_to_closed_cubic_path() {
    let points: Vec<_> = (0..32)
        .map(|index| {
            let angle = core::f64::consts::TAU * index as f64 / 32.0;
            Point2::new(angle.cos() * 10.0, angle.sin() * 10.0)
        })
        .collect();
    let source = path_from_points(&points, true);
    let fitted = fit_path_curves(&source, 0.1, Tolerance::default()).unwrap();

    assert!(fitted.subpaths()[0].is_closed());
    assert!(
        fitted.subpaths()[0]
            .segments()
            .iter()
            .all(|segment| matches!(segment, Segment::Cubic(_)))
    );
    assert!(fitted.segment_count() < source.segment_count());
    assert!(max_sample_distance(&fitted, &points, Tolerance::default()) <= 0.12);
}

#[test]
fn curve_fitting_is_deterministic() {
    let points: Vec<_> = (0..=60)
        .map(|index| {
            let x = index as f64 * 0.2;
            Point2::new(x, (x * 1.3).sin() * 3.0)
        })
        .collect();
    let path = path_from_points(&points, false);

    let first = fit_path_curves(&path, 0.05, Tolerance::default()).unwrap();
    let second = fit_path_curves(&path, 0.05, Tolerance::default()).unwrap();

    assert_eq!(first, second);
}

#[test]
fn invalid_cleanup_and_fit_tolerances_are_rejected() {
    let path = path_from_points(&[Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)], false);

    assert_eq!(
        simplify_path(&path, 0.0, Tolerance::default()),
        Err(CoreError::InvalidGeometry)
    );
    assert_eq!(
        fit_path_curves(&path, f64::NAN, Tolerance::default()),
        Err(CoreError::InvalidNumber)
    );
    assert_eq!(
        cleanup_path(
            &path,
            CleanupOptions {
                point_tolerance: -1.0,
                collinear_tolerance: 0.0,
            },
            Tolerance::default(),
        ),
        Err(CoreError::InvalidNumber)
    );
}
