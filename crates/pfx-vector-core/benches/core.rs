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
}

criterion_group!(benches, bench_core);
criterion_main!(benches);
