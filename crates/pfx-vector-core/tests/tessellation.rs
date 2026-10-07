use pfx_vector_core::*;

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

fn rect_path(x: f64, y: f64, width: f64, height: f64) -> Path {
    rect_to_path(Rect::new(x, y, width, height).unwrap()).unwrap()
}

#[test]
fn rectangle_tessellates_to_two_triangles() {
    let path = rect_path(0.0, 0.0, 10.0, 10.0);
    let mesh = tessellate_fill(&path, FillRule::NonZero, Tolerance::default()).unwrap();

    assert_eq!(mesh.triangle_count(), 2);
    assert!((mesh_area(&mesh) - 100.0).abs() < 1.0e-9);
}

#[test]
fn concave_polygon_triangulates_without_area_loss() {
    let polygon = Polygon::new(vec![
        Point2::new(0.0, 0.0),
        Point2::new(10.0, 0.0),
        Point2::new(10.0, 10.0),
        Point2::new(5.0, 5.0),
        Point2::new(0.0, 10.0),
    ])
    .unwrap();
    let path = polygon_to_path(&polygon).unwrap();
    let mesh = tessellate_fill(&path, FillRule::NonZero, Tolerance::default()).unwrap();

    assert_eq!(mesh.triangle_count(), 3);
    assert!((mesh_area(&mesh) - 75.0).abs() < 1.0e-8);
}

#[test]
fn hole_is_removed_from_mesh_area() {
    let outer = rect_path(0.0, 0.0, 20.0, 20.0);
    let inner = rect_path(5.0, 5.0, 10.0, 10.0);
    let ring = boolean_difference(&outer, &inner, Tolerance::default()).unwrap();
    let mesh = tessellate_fill(&ring, FillRule::NonZero, Tolerance::default()).unwrap();

    assert!((mesh_area(&mesh) - 300.0).abs() < 1.0e-7);
}

#[test]
fn evenodd_same_direction_contours_create_hole() {
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
    let path = builder.finish().unwrap();

    let mesh = tessellate_fill(&path, FillRule::EvenOdd, Tolerance::default()).unwrap();
    assert!((mesh_area(&mesh) - 300.0).abs() < 1.0e-7);
}

#[test]
fn nonzero_same_direction_nested_contour_is_redundant() {
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
    let path = builder.finish().unwrap();

    let mesh = tessellate_fill(&path, FillRule::NonZero, Tolerance::default()).unwrap();
    assert!((mesh_area(&mesh) - 400.0).abs() < 1.0e-7);
}

#[test]
fn disjoint_contours_share_one_mesh() {
    let a = rect_path(0.0, 0.0, 5.0, 5.0);
    let b = rect_path(10.0, 0.0, 5.0, 5.0);
    let combined = boolean_union(&a, &b, Tolerance::default()).unwrap();
    let mesh = tessellate_fill(&combined, FillRule::NonZero, Tolerance::default()).unwrap();

    assert_eq!(mesh.triangle_count(), 4);
    assert!((mesh_area(&mesh) - 50.0).abs() < 1.0e-8);
}

#[test]
fn circle_tessellation_converges_with_flatness() {
    let circle = circle_to_path(Circle::new(Point2::new(0.0, 0.0), 10.0).unwrap()).unwrap();
    let tolerance = Tolerance {
        flatness: 0.01,
        ..Tolerance::default()
    };
    let mesh = tessellate_fill(&circle, FillRule::NonZero, tolerance).unwrap();

    assert!((mesh_area(&mesh) - core::f64::consts::PI * 100.0).abs() < 1.0);
    assert!(mesh.triangle_count() > 8);
}

#[test]
fn stroke_tessellation_reuses_outline_geometry() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap();
    let path = builder.finish().unwrap();
    let style = StrokeStyle {
        width: 4.0,
        cap: StrokeCap::Butt,
        join: StrokeJoin::Miter,
        miter_limit: 4.0,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };

    let mesh = tessellate_stroke(&path, &style, Tolerance::default()).unwrap();
    assert!((mesh_area(&mesh) - 40.0).abs() < 1.0e-7);
}

#[test]
fn reversed_path_has_same_tessellated_area() {
    let path = rect_path(0.0, 0.0, 10.0, 6.0);
    let forward = tessellate_fill(&path, FillRule::NonZero, Tolerance::default()).unwrap();
    let reverse =
        tessellate_fill(&path.reversed(), FillRule::NonZero, Tolerance::default()).unwrap();

    assert!((mesh_area(&forward) - mesh_area(&reverse)).abs() < 1.0e-9);
}

#[test]
fn large_coordinates_remain_stable() {
    let offset = 1.0e9;
    let path = rect_path(offset, offset, 100.0, 50.0);
    let mesh = tessellate_fill(&path, FillRule::NonZero, Tolerance::default()).unwrap();

    assert!((mesh_area(&mesh) - 5000.0).abs() < 1.0e-3);
}

#[test]
fn open_fill_subpaths_are_explicitly_rejected() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(10.0, 0.0))
        .unwrap()
        .line_to(Point2::new(5.0, 10.0))
        .unwrap();
    let path = builder.finish().unwrap();

    assert_eq!(
        tessellate_fill(&path, FillRule::NonZero, Tolerance::default()),
        Err(CoreError::UnsupportedCase)
    );
}
