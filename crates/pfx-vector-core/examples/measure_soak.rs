//! Bounded repeated-build soak probe, independent of allocator implementation.
//! Linux: cargo run --release -p pfx-vector-core --example measure_soak
use pfx_vector_core::{PathBuilder, PathMeasureIndex, Point2, Tolerance};
use std::hint::black_box;
use std::time::Instant;

fn resident_kib() -> Option<usize> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("VmRSS:"))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

fn main() {
    let mut builder = PathBuilder::new();
    builder.move_to(Point2::new(0.0, 0.0)).unwrap();
    const COUNT: usize = 32;
    for i in 0..COUNT {
        let x = i as f64 * 10.0;
        builder
            .cubic_to(
                Point2::new(x + 2.0, 5.0),
                Point2::new(x + 8.0, -5.0),
                Point2::new(x + 10.0, 0.0),
            )
            .unwrap();
    }
    let path = builder.finish().unwrap();
    println!("round,segments,build_ms,query_ms,resident_kib,samples");
    for round in 1..=12 {
        let started = Instant::now();
        let index = PathMeasureIndex::build(&path, Tolerance::default()).unwrap();
        let build_ms = started.elapsed().as_secs_f64() * 1000.0;
        let started = Instant::now();
        for i in 0..1000 {
            black_box(
                index
                    .point_at_length(index.total_length() * i as f64 / 999.0)
                    .unwrap(),
            );
        }
        let query_ms = started.elapsed().as_secs_f64() * 1000.0;
        let samples = index.sample_count();
        drop(index);
        let rss = resident_kib()
            .map_or_else(|| "unavailable".to_owned(), |value| value.to_string());
        println!("{round},{COUNT},{build_ms:.3},{query_ms:.3},{rss},{samples}");
    }
}
