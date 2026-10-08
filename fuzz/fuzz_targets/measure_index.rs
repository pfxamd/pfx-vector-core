#![no_main]
use libfuzzer_sys::fuzz_target;
use pfx_vector_core::{PathMeasureIndex, Tolerance};

fuzz_target!(|data: &[u8]| {
    // Limit parser input and geometry work to keep fuzz throughput meaningful.
    if data.len() > 4096 {
        return;
    }
    let Ok(source) = core::str::from_utf8(data) else {
        return;
    };
    let Ok(path) = pfx_vector_svg::parse_path(source) else {
        return;
    };
    if path.segment_count() > 128 {
        return;
    }
    let tolerance = Tolerance::default();
    if let Ok(index) = PathMeasureIndex::build(&path, tolerance) {
        let total = index.total_length();
        assert!(total.is_finite() && total >= 0.0);
        if index.segment_count() > 0 {
            for fraction in [0.0, 0.25, 0.5, 0.75, 1.0] {
                if let Ok((point, location)) = index.point_at_length(total * fraction) {
                    assert!(point.x.is_finite() && point.y.is_finite());
                    assert!(location.t.is_finite());
                }
            }
        }
    }
});
