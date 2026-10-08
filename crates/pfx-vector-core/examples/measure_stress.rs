//! Standalone, repeatable scaling probe. Not a CI performance threshold.
//! Run on Linux: cargo run --release -p pfx-vector-core --example measure_stress
use pfx_vector_core::{Angle, EllipticalArc, PathBuilder, PathMeasureIndex, Point2, Tolerance};
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

    // Curved workloads use bounded sizes because numerical integration is
    // considerably more expensive than line indexing.
    for count in [32_usize, 128, 512] {
        let mut builder = PathBuilder::new();
        builder.move_to(Point2::new(0.0, 0.0)).unwrap();
        for i in 0..count {
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
        let started = Instant::now();
        let index = PathMeasureIndex::build(&path, Tolerance::default()).unwrap();
        let build_ms = started.elapsed().as_secs_f64() * 1000.0;
        const QUERIES: usize = 10_000;
        let started = Instant::now();
        for i in 0..QUERIES {
            let distance = index.total_length() * i as f64 / (QUERIES - 1) as f64;
            black_box(index.point_at_length(black_box(distance)).unwrap());
        }
        let query_ms = started.elapsed().as_secs_f64() * 1000.0;
        let memory =
            peak_rss_kib().map_or_else(|| "unavailable".to_owned(), |value| value.to_string());
        println!(
            "cubic_index,{count},{QUERIES},{build_ms:.3},{query_ms:.3},{memory},{}",
            index.sample_count()
        );
    }
    for count in [16_usize, 64, 256] {
        let mut builder = PathBuilder::new();
        for i in 0..count {
            let arc = EllipticalArc::new(
                Point2::new(i as f64 * 100.0, 0.0),
                30.0,
                12.0,
                Angle::degrees(15.0),
                Angle::degrees(0.0),
                Angle::degrees(180.0),
            );
            builder.move_to(arc.point_at(0.0)).unwrap();
            builder.arc_to(arc).unwrap();
        }
        let path = builder.finish().unwrap();
        let started = Instant::now();
        let index = PathMeasureIndex::build(&path, Tolerance::default()).unwrap();
        let build_ms = started.elapsed().as_secs_f64() * 1000.0;
        const QUERIES: usize = 10_000;
        let started = Instant::now();
        for i in 0..QUERIES {
            let distance = index.total_length() * i as f64 / (QUERIES - 1) as f64;
            black_box(index.point_at_length(black_box(distance)).unwrap());
        }
        let query_ms = started.elapsed().as_secs_f64() * 1000.0;
        let memory =
            peak_rss_kib().map_or_else(|| "unavailable".to_owned(), |value| value.to_string());
        println!(
            "arc_index,{count},{QUERIES},{build_ms:.3},{query_ms:.3},{memory},{}",
            index.sample_count()
        );
    }
}
