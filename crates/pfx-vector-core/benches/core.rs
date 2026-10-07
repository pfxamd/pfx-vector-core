use criterion::{
    black_box, criterion_group, criterion_main, Criterion
};
use pfx_vector_core::{
    CubicBezier, Point2, Segment, Tolerance, flatten_segment, segment_length
};
fn bench_curves(c: &mut Criterion) {
    let curve = Segment::Cubic(CubicBezier::new(Point2::new(0.0, 0.0), Point2::new(100.0, 300.0),
    Point2::new(300.0, -100.0), Point2::new(500.0, 200.0)));
    let tol = Tolerance::default();
    c.bench_function("cubic_length", |b|b.iter(|| segment_length(black_box(curve), black_box(tol)).unwrap()));
    c.bench_function("cubic_flatten", |b|b.iter(|| flatten_segment(black_box(curve), black_box(tol)).unwrap()));
}
criterion_group!(benches, bench_curves);
criterion_main!(benches);
