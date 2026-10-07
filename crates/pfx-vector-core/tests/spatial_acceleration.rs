use pfx_vector_core::*;

fn bounds(x0: f64, y0: f64, x1: f64, y1: f64) -> Bounds {
    Bounds::Finite {
        min: Point2::new(x0, y0),
        max: Point2::new(x1, y1),
    }
}

fn brute_self_pairs(items: &[Bounds], padding: f64) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    for left in 0..items.len() {
        for right in left + 1..items.len() {
            if items[left].expanded(padding).intersects(items[right]) {
                result.push((left, right));
            }
        }
    }
    result
}

fn brute_cross_pairs(left: &[Bounds], right: &[Bounds], padding: f64) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    for (left_index, &left_bounds) in left.iter().enumerate() {
        for (right_index, &right_bounds) in right.iter().enumerate() {
            if left_bounds.expanded(padding).intersects(right_bounds) {
                result.push((left_index, right_index));
            }
        }
    }
    result
}

fn dense_wave(count: usize) -> Path {
    let mut builder = PathBuilder::new();
    builder.move_to(Point2::new(0.0, 0.0)).unwrap();
    for index in 1..=count {
        let x = index as f64 * 0.1;
        builder
            .line_to(Point2::new(x, (x * 0.31).sin() * 8.0))
            .unwrap();
    }
    builder.finish().unwrap()
}

#[test]
fn spatial_query_returns_only_overlapping_items_in_stable_order() {
    let items = vec![
        bounds(20.0, 0.0, 25.0, 5.0),
        bounds(0.0, 0.0, 5.0, 5.0),
        bounds(4.0, 4.0, 8.0, 8.0),
        Bounds::Empty,
        bounds(6.0, -2.0, 7.0, 2.0),
    ];
    let index = SpatialIndex::new(&items);
    let query = bounds(3.0, 1.0, 6.5, 6.0);

    assert_eq!(index.query_bounds(query, 0.0).unwrap(), vec![1, 2, 4]);
    assert_eq!(index.query_bounds(query, 0.0).unwrap(), vec![1, 2, 4]);
    assert_eq!(index.len(), 4);
    assert!(!index.is_empty());
}

#[test]
fn self_candidate_pairs_match_brute_force() {
    let items = vec![
        bounds(0.0, 0.0, 5.0, 5.0),
        bounds(4.0, 3.0, 8.0, 7.0),
        bounds(10.0, 0.0, 12.0, 2.0),
        bounds(11.5, 1.5, 14.0, 3.0),
        bounds(-3.0, -3.0, -1.0, -1.0),
    ];
    let padding = 0.25;

    assert_eq!(
        spatial_self_candidate_pairs(&items, padding).unwrap(),
        brute_self_pairs(&items, padding)
    );
}

#[test]
fn cross_candidate_pairs_match_brute_force() {
    let left = vec![
        bounds(0.0, 0.0, 4.0, 4.0),
        bounds(10.0, 10.0, 20.0, 20.0),
        bounds(-10.0, -2.0, -5.0, 2.0),
    ];
    let right = vec![
        bounds(3.0, 2.0, 6.0, 5.0),
        bounds(19.0, 0.0, 21.0, 11.0),
        bounds(-6.0, -1.0, -4.0, 1.0),
        bounds(100.0, 100.0, 101.0, 101.0),
    ];

    assert_eq!(
        spatial_cross_candidate_pairs(&left, &right, 0.0).unwrap(),
        brute_cross_pairs(&left, &right, 0.0)
    );
}

#[test]
fn sparse_large_input_avoids_quadratic_candidate_explosion() {
    let items: Vec<_> = (0..5000)
        .map(|index| {
            let x = index as f64 * 10.0;
            bounds(x, 0.0, x + 1.0, 1.0)
        })
        .collect();

    let pairs = spatial_self_candidate_pairs(&items, 0.0).unwrap();

    assert!(pairs.is_empty());
}

#[test]
fn path_spatial_fill_matches_regular_classifier() {
    let outer = rect_to_path(Rect::new(0.0, 0.0, 30.0, 30.0).unwrap()).unwrap();
    let inner = rect_to_path(Rect::new(8.0, 8.0, 14.0, 14.0).unwrap()).unwrap();
    let path = boolean_difference(&outer, &inner, Tolerance::default()).unwrap();
    let index = PathSpatialIndex::build(&path, Tolerance::default()).unwrap();

    for x in -2..=32 {
        for y in -2..=32 {
            let point = Point2::new(f64::from(x) + 0.31, f64::from(y) + 0.17);
            assert_eq!(
                index.classify_point(point, FillRule::NonZero).unwrap(),
                classify_point(&path, point, FillRule::NonZero, Tolerance::default()).unwrap(),
                "point=({},{})",
                point.x,
                point.y
            );
        }
    }
}

#[test]
fn path_spatial_stroke_matches_regular_hit_test() {
    let path = dense_wave(250);
    let style = StrokeStyle {
        width: 2.5,
        ..StrokeStyle::default()
    };
    let index = PathSpatialIndex::build(&path, Tolerance::default()).unwrap();

    for x in 0..=25 {
        for y in -10..=10 {
            let point = Point2::new(f64::from(x), f64::from(y));
            assert_eq!(
                index.stroke_contains_point(&style, point).unwrap(),
                stroke_contains_point(&path, &style, point, Tolerance::default()).unwrap(),
                "point=({},{})",
                point.x,
                point.y
            );
        }
    }
}

#[test]
fn path_spatial_nearest_matches_regular_nearest_on_open_path() {
    let path = dense_wave(400);
    let index = PathSpatialIndex::build(&path, Tolerance::default()).unwrap();

    for point in [
        Point2::new(1.0, 20.0),
        Point2::new(10.0, 0.0),
        Point2::new(22.0, -15.0),
        Point2::new(50.0, 3.0),
    ] {
        let indexed = index.closest_point(point).unwrap();
        let regular = closest_point(&path, point, Tolerance::default()).unwrap();

        assert!((indexed.distance - regular.distance).abs() < 1.0e-9);
        assert!(indexed.point.distance_to(regular.point) < 1.0e-9);
    }
}

#[test]
fn spatial_queries_remain_stable_at_large_coordinates() {
    let offset = 1.0e9;
    let items = vec![
        bounds(offset, offset, offset + 10.0, offset + 10.0),
        bounds(offset + 5.0, offset + 5.0, offset + 15.0, offset + 15.0),
        bounds(offset + 100.0, offset, offset + 110.0, offset + 10.0),
    ];

    assert_eq!(
        spatial_self_candidate_pairs(&items, Tolerance::default().absolute).unwrap(),
        vec![(0, 1)]
    );

    let path = rect_to_path(Rect::new(offset, offset, 20.0, 10.0).unwrap()).unwrap();
    let index = PathSpatialIndex::build(&path, Tolerance::default()).unwrap();
    assert!(
        index
            .contains_point(Point2::new(offset + 5.0, offset + 5.0), FillRule::NonZero)
            .unwrap()
    );
}

#[test]
fn invalid_spatial_padding_is_rejected() {
    let items = vec![bounds(0.0, 0.0, 1.0, 1.0)];
    let index = SpatialIndex::new(&items);

    assert_eq!(
        index.query_bounds(items[0], -1.0),
        Err(CoreError::InvalidGeometry)
    );
    assert_eq!(
        spatial_self_candidate_pairs(&items, f64::NAN),
        Err(CoreError::InvalidNumber)
    );
}

#[test]
fn path_spatial_nearest_refines_against_source_curve() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .quad_to(Point2::new(5.0, 10.0), Point2::new(10.0, 0.0))
        .unwrap();
    let path = builder.finish().unwrap();
    let tolerance = Tolerance {
        flatness: 100.0,
        ..Tolerance::default()
    };
    let index = PathSpatialIndex::build(&path, tolerance).unwrap();

    let result = index.closest_point(Point2::new(5.0, 8.0)).unwrap();

    assert_eq!(result.location.segment_index, 0);
    assert!((result.location.t - 0.5).abs() <= 1.0e-9);
    assert!(result.point.distance_to(Point2::new(5.0, 5.0)) <= 1.0e-8);
    assert!((result.distance - 3.0).abs() <= 1.0e-8);
}
