use pfx_vector_core::{PathMeasureIndex, Tolerance};
use pfx_vector_svg::{SerializeOptions, parse_path, serialize_path};

#[test]
fn svg_to_core_measure_round_trip_is_finite() {
    let cases = [
        "M0 0 L3 4 L10 4",
        "M0 0 C15 35 55 -20 80 0",
        "M5 2 Q30 48 60 2",
        "M0 0 L10 0 L10 10 Z",
        "M0 0 A30 12 15 0 1 60 0",
    ];
    for source in cases {
        let original = parse_path(source).unwrap();
        let index = PathMeasureIndex::build(&original, Tolerance::default()).unwrap();
        let total = index.total_length();
        assert!(total.is_finite());
        assert!(total >= 0.0);
        // A near-degenerate SVG arc is sensitive to rounding of corrected radii.
        // Use high-precision serialization for geometry-preserving round trips.
        let svg = serialize_path(
            &original,
            SerializeOptions {
                precision: 17,
                ..SerializeOptions::default()
            },
        );
        let restored = parse_path(&svg).unwrap();
        let rebuilt = PathMeasureIndex::build(&restored, Tolerance::default()).unwrap();
        let threshold = 1.0e-6_f64.max(total * 1.0e-7);
        assert!(
            (total - rebuilt.total_length()).abs() <= threshold,
            "source={source}, serialized={svg}, original={total}, restored={}",
            rebuilt.total_length()
        );
        for fraction in [0.0, 0.25, 0.5, 0.75, 1.0] {
            if index.segment_count() > 0 {
                let (_, location) = index.point_at_length(total * fraction).unwrap();
                assert!(location.t.is_finite());
            }
        }
    }
}

#[test]
fn malformed_svg_never_produces_valid_measure_index() {
    for source in ["M0 0 L", "M0 0 C10 20", "M0 0 LNaN 1", "M0 0 L1e999 0"] {
        assert!(parse_path(source).is_err());
    }
}
