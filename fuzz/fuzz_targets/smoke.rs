#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Bound work per iteration, including source length and parsed complexity.
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
    let tolerance = pfx_vector_core::Tolerance::default();
    let _ = pfx_vector_core::path_length(&path, tolerance);
    let _ = pfx_vector_core::flatten_path(&path, tolerance);
    let output =
        pfx_vector_svg::serialize_path(&path, pfx_vector_svg::SerializeOptions::default());
    assert!(pfx_vector_svg::parse_path(&output).is_ok());
});
