#![forbid(unsafe_code)]
use pfx_vector_core::{
    BooleanOperation, Bounds, FillRule, Point2, Tolerance, boolean_paths, contains_point,
    flatten_path, intersect_segments, path_length, point_at_length,
};
use pfx_vector_svg::{SerializeOptions, parse_path, serialize_path};
use wasm_bindgen::prelude::*;
fn js_err<E: core::fmt::Display>(e: E) -> JsValue {
    JsValue::from_str(&e.to_string())
}
#[wasm_bindgen]
pub fn core_version() -> String {
    pfx_vector_core::VERSION.to_owned()
}
#[wasm_bindgen]
pub fn validate_path(data: &str) -> bool {
    parse_path(data).is_ok()
}
#[wasm_bindgen]
pub fn path_length_svg(data: &str) -> Result<f64, JsValue> {
    let p = parse_path(data).map_err(js_err)?;
    path_length(&p, Tolerance::default()).map_err(js_err)
}
#[wasm_bindgen]
pub fn path_bounds_svg(data: &str) -> Result<String, JsValue> {
    let p = parse_path(data).map_err(js_err)?;
    let mut b = Bounds::Empty;
    for s in p.subpaths() {
        for seg in s.segments() {
            b = b.union(seg.bounds())
        }
    }
    Ok(match b {
        Bounds::Empty => "null".to_owned(),
        Bounds::Finite { min, max } => format!(
            "{{\"minX\":{},\"minY\":{},\"maxX\":{},\"maxY\":{}}}",
            min.x, min.y, max.x, max.y
        ),
    })
}
#[wasm_bindgen]
pub fn point_at_length_svg(data: &str, distance: f64) -> Result<String, JsValue> {
    let p = parse_path(data).map_err(js_err)?;
    let (pt, loc) = point_at_length(&p, distance, Tolerance::default()).map_err(js_err)?;
    Ok(format!(
        "{{\"x\":{},\"y\":{},\"subpath\":{},\"segment\":{},\"t\":{},\"distance\":{}}}",
        pt.x, pt.y, loc.subpath_index, loc.segment_index, loc.t, loc.distance
    ))
}
#[wasm_bindgen]
pub fn hit_test_fill_svg(data: &str, x: f64, y: f64, even_odd: bool) -> Result<bool, JsValue> {
    let p = parse_path(data).map_err(js_err)?;
    contains_point(
        &p,
        Point2::new(x, y),
        if even_odd {
            FillRule::EvenOdd
        } else {
            FillRule::NonZero
        },
        Tolerance::default(),
    )
    .map_err(js_err)
}
#[wasm_bindgen]
pub fn flatten_path_svg(data: &str, flatness: f64) -> Result<String, JsValue> {
    let p = parse_path(data).map_err(js_err)?;
    let tol = Tolerance {
        flatness: flatness.max(1e-12),
        ..Tolerance::default()
    };
    let flat = flatten_path(&p, tol).map_err(js_err)?;
    let mut b = pfx_vector_core::PathBuilder::with_tolerance(tol);
    for sub in flat {
        if let Some(&first) = sub.points.first() {
            b.move_to(first).map_err(js_err)?;
            for &q in &sub.points[1..] {
                b.line_to(q).map_err(js_err)?;
            }
            if sub.closed {
                b.close().map_err(js_err)?;
            }
        }
    }
    let out = b.finish().map_err(js_err)?;
    Ok(serialize_path(&out, SerializeOptions::default()))
}
#[wasm_bindgen]
pub fn normalize_path_svg(data: &str) -> Result<String, JsValue> {
    let p = parse_path(data).map_err(js_err)?;
    Ok(serialize_path(&p, SerializeOptions::default()))
}
#[wasm_bindgen]
pub fn intersect_paths_svg(a: &str, b: &str) -> Result<String, JsValue> {
    let pa = parse_path(a).map_err(js_err)?;
    let pb = parse_path(b).map_err(js_err)?;
    let tol = Tolerance::default();
    let mut points = Vec::new();
    for sa in pa.subpaths() {
        for &ga in sa.segments() {
            for sb in pb.subpaths() {
                for &gb in sb.segments() {
                    let r = match intersect_segments(ga, gb, tol) {
                        Ok(v) => v,
                        Err(pfx_vector_core::CoreError::UnsupportedCase) => continue,
                        Err(e) => return Err(js_err(e)),
                    };
                    for hit in r.intersections {
                        if let pfx_vector_core::Intersection::Point(p) = hit {
                            points.push(p.point)
                        }
                    }
                }
            }
        }
    }
    let body = points
        .iter()
        .map(|p| format!("{{\"x\":{},\"y\":{}}}", p.x, p.y))
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!("[{body}]"))
}

fn boolean_svg(a: &str, b: &str, operation: BooleanOperation) -> Result<String, JsValue> {
    let path_a = parse_path(a).map_err(js_err)?;
    let path_b = parse_path(b).map_err(js_err)?;
    let result =
        boolean_paths(&path_a, &path_b, operation, Tolerance::default()).map_err(js_err)?;
    Ok(serialize_path(&result, SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn boolean_union_svg(a: &str, b: &str) -> Result<String, JsValue> {
    boolean_svg(a, b, BooleanOperation::Union)
}

#[wasm_bindgen]
pub fn boolean_intersection_svg(a: &str, b: &str) -> Result<String, JsValue> {
    boolean_svg(a, b, BooleanOperation::Intersection)
}

#[wasm_bindgen]
pub fn boolean_difference_svg(a: &str, b: &str) -> Result<String, JsValue> {
    boolean_svg(a, b, BooleanOperation::Difference)
}

#[wasm_bindgen]
pub fn boolean_xor_svg(a: &str, b: &str) -> Result<String, JsValue> {
    boolean_svg(a, b, BooleanOperation::Xor)
}
