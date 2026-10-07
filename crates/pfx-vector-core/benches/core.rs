use criterion::{black_box,criterion_group,criterion_main,Criterion};
use pfx_vector_core::*;
fn bench_core(c:&mut Criterion){let curve=CubicBezier::new(Point2::new(0.0,0.0),Point2::new(100.0,300.0),Point2::new(200.0,-300.0),Point2::new(400.0,0.0));c.bench_function("cubic bounds",|b|b.iter(||black_box(curve).bounds()));let mut pb=PathBuilder::new();pb.move_to(Point2::new(0.0,0.0)).unwrap().cubic_to(curve.p1,curve.p2,curve.p3).unwrap();let path=pb.finish().unwrap();c.bench_function("path length",|b|b.iter(||path_length(black_box(&path),Tolerance::default()).unwrap()));}
criterion_group!(benches,bench_core);criterion_main!(benches);
