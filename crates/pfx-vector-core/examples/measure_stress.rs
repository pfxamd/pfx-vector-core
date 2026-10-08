//! Standalone, repeatable scaling probe. Not a CI performance threshold.
//! Run on Linux: cargo run --release -p pfx-vector-core --example measure_stress
use pfx_vector_core::{PathBuilder, PathMeasureIndex, Point2, Tolerance};
use std::hint::black_box;
use std::time::Instant;

fn peak_rss_kib() -> Option<usize> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("VmHWM:"))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

fn main() {
    println!("workload,segments,queries,build_ms,query_ms,peak_rss_kib,samples");
    for count in [4096_usize, 16384, 65536] {
        let mut builder = PathBuilder::new();
        builder.move_to(Point2::new(0.0, 0.0)).unwrap();
        for i in 1..=count {
            builder.line_to(Point2::new(i as f64, 0.0)).unwrap();
        }
        let path = builder.finish().unwrap();

        let start = Instant::now();
        let index = PathMeasureIndex::build(&path, Tolerance::default()).unwrap();
        let build_ms = start.elapsed().as_secs_f64() * 1000.0;

        const QUERIES: usize = 10_000;
        let start = Instant::now();
        for i in 0..QUERIES {
            let distance = index.total_length() * i as f64 / (QUERIES - 1) as f64;
            black_box(index.point_at_length(black_box(distance)).unwrap());
        }
        let query_ms = start.elapsed().as_secs_f64() * 1000.0;
        let memory =
            peak_rss_kib().map_or_else(|| "unavailable".to_owned(), |value| value.to_string());
        println!(
            "line_index,{count},{QUERIES},{build_ms:.3},{query_ms:.3},{memory},{}",
            index.sample_count()
        );
    }
}
