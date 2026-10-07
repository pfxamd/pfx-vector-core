use criterion::{Criterion, black_box, criterion_group, criterion_main};
use pfx_vector_core::*;

fn bench_core(c: &mut Criterion) {
    let curve = CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(100.0, 300.0),
        Point2::new(200.0, -300.0),
        Point2::new(400.0, 0.0),
    );
    c.bench_function("cubic bounds", |b| b.iter(|| black_box(curve).bounds()));

    let mut builder = PathBuilder::new();
    builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(curve.p1, curve.p2, curve.p3)
        .unwrap();
    let path = builder.finish().unwrap();

    c.bench_function("path length", |b| {
        b.iter(|| path_length(black_box(&path), Tolerance::default()).unwrap())
    });

    let intersection_a = Segment::Cubic(CubicBezier::new(
        Point2::new(0.0, 0.0),
        Point2::new(100.0, 300.0),
        Point2::new(200.0, -300.0),
        Point2::new(400.0, 0.0),
    ));
    let intersection_b = Segment::Cubic(CubicBezier::new(
        Point2::new(0.0, 100.0),
        Point2::new(120.0, -250.0),
        Point2::new(280.0, 350.0),
        Point2::new(400.0, -100.0),
    ));
    let tolerance = Tolerance::default();

    c.bench_function("advanced cubic intersection", |b| {
        b.iter(|| {
            intersect_segments(
                black_box(intersection_a),
                black_box(intersection_b),
                black_box(tolerance),
            )
            .unwrap()
        })
    });

    let boolean_a = rect_to_path(Rect::new(0.0, 0.0, 100.0, 100.0).unwrap()).unwrap();
    let boolean_b = rect_to_path(Rect::new(50.0, 25.0, 100.0, 100.0).unwrap()).unwrap();

    c.bench_function("boolean union rectangles", |b| {
        b.iter(|| {
            boolean_union(
                black_box(&boolean_a),
                black_box(&boolean_b),
                black_box(tolerance),
            )
            .unwrap()
        })
    });

    c.bench_function("offset rectangle", |b| {
        b.iter(|| {
            offset_path(
                black_box(&boolean_a),
                black_box(10.0),
                black_box(OffsetStyle::default()),
                black_box(tolerance),
            )
            .unwrap()
        })
    });

    let mut outline_builder = PathBuilder::new();
    outline_builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .cubic_to(
            Point2::new(25.0, 50.0),
            Point2::new(75.0, -50.0),
            Point2::new(100.0, 0.0),
        )
        .unwrap();
    let outline_source = outline_builder.finish().unwrap();
    let outline_style = StrokeStyle {
        width: 8.0,
        cap: StrokeCap::Round,
        join: StrokeJoin::Round,
        miter_limit: 4.0,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };

    c.bench_function("outline cubic path", |b| {
        b.iter(|| {
            outline_path(
                black_box(&outline_source),
                black_box(&outline_style),
                black_box(tolerance),
            )
            .unwrap()
        })
    });

    c.bench_function("tessellate rectangle fill", |b| {
        b.iter(|| {
            tessellate_fill(
                black_box(&boolean_a),
                black_box(FillRule::NonZero),
                black_box(tolerance),
            )
            .unwrap()
        })
    });

    let circle = circle_to_path(Circle::new(Point2::new(0.0, 0.0), 100.0).unwrap()).unwrap();
    let mesh_tolerance = Tolerance {
        flatness: 0.1,
        ..tolerance
    };

    c.bench_function("tessellate circle fill", |b| {
        b.iter(|| {
            tessellate_fill(
                black_box(&circle),
                black_box(FillRule::NonZero),
                black_box(mesh_tolerance),
            )
            .unwrap()
        })
    });

    c.bench_function("tessellate cubic stroke", |b| {
        b.iter(|| {
            tessellate_stroke(
                black_box(&outline_source),
                black_box(&outline_style),
                black_box(mesh_tolerance),
            )
            .unwrap()
        })
    });

    let mut dense_builder = PathBuilder::new();
    dense_builder.move_to(Point2::new(0.0, 0.0)).unwrap();
    for index in 1..=1000 {
        let x = index as f64 * 0.02;
        dense_builder
            .line_to(Point2::new(x, (x * 0.9).sin() * 4.0))
            .unwrap();
    }
    let dense_path = dense_builder.finish().unwrap();

    c.bench_function("cleanup dense polyline", |b| {
        b.iter(|| {
            cleanup_path(
                black_box(&dense_path),
                black_box(CleanupOptions::default()),
                black_box(tolerance),
            )
            .unwrap()
        })
    });

    c.bench_function("simplify dense polyline", |b| {
        b.iter(|| {
            simplify_path(
                black_box(&dense_path),
                black_box(0.05),
                black_box(tolerance),
            )
            .unwrap()
        })
    });

    c.bench_function("fit dense polyline to cubics", |b| {
        b.iter(|| {
            fit_path_curves(
                black_box(&dense_path),
                black_box(0.05),
                black_box(tolerance),
            )
            .unwrap()
        })
    });

    let mut self_crossing_builder = PathBuilder::new();
    self_crossing_builder
        .move_to(Point2::new(0.0, 0.0))
        .unwrap()
        .line_to(Point2::new(100.0, 100.0))
        .unwrap()
        .line_to(Point2::new(0.0, 100.0))
        .unwrap()
        .line_to(Point2::new(100.0, 0.0))
        .unwrap()
        .close()
        .unwrap();
    let self_crossing_path = self_crossing_builder.finish().unwrap();

    c.bench_function("normalize self-intersecting contour", |b| {
        b.iter(|| {
            normalize_self_intersections(
                black_box(&self_crossing_path),
                black_box(FillRule::EvenOdd),
                black_box(tolerance),
            )
            .unwrap()
        })
    });

    let sparse_bounds: Vec<_> = (0..10_000)
        .map(|index| {
            let x = index as f64 * 10.0;
            Bounds::from_points(&[
                Point2::new(x, 0.0),
                Point2::new(x + 1.0, 1.0),
            ])
        })
        .collect();

    c.bench_function("spatial index build 10000 bounds", |b| {
        b.iter(|| SpatialIndex::new(black_box(&sparse_bounds)))
    });

    c.bench_function("spatial self candidates 10000 sparse bounds", |b| {
        b.iter(|| {
            spatial_self_candidate_pairs(
                black_box(&sparse_bounds),
                black_box(tolerance.absolute),
            )
            .unwrap()
        })
    });

    let dense_index = PathSpatialIndex::build(&dense_path, tolerance).unwrap();
    c.bench_function("indexed nearest dense path", |b| {
        b.iter(|| {
            dense_index
                .closest_point(black_box(Point2::new(10.0, 12.0)))
                .unwrap()
        })
    });
}

criterion_group!(benches, bench_core);
criterion_main!(benches);
