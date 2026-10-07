#![forbid(unsafe_code)]
use pfx_vector_core::{
    BooleanOperation, Bounds, CleanupOptions, ContourSliceMode, FillRule, Mesh2D, OffsetStyle,
    Point2, StrokeCap, StrokeJoin, StrokeStyle, Tolerance, boolean_paths, cleanup_path,
    contains_point, contour_length, dash_path, fit_path_curves, flatten_path, intersect_segments,
    normalize_self_intersections, offset_path, outline_path, path_length, point_at_length,
    simplify_path, slice_contour, spatial_cross_candidate_pairs, tessellate_fill,
    tessellate_stroke,
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
pub fn contour_length_svg(data: &str, subpath_index: u32) -> Result<f64, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let subpath = path
        .subpaths()
        .get(subpath_index as usize)
        .ok_or_else(|| JsValue::from_str("subpath index out of range"))?;
    contour_length(subpath, Tolerance::default()).map_err(js_err)
}

#[wasm_bindgen]
pub fn slice_contour_svg(
    data: &str,
    subpath_index: u32,
    start_distance: f64,
    end_distance: f64,
    wrap: bool,
) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let subpath = path
        .subpaths()
        .get(subpath_index as usize)
        .ok_or_else(|| JsValue::from_str("subpath index out of range"))?;
    let sliced = slice_contour(
        subpath,
        start_distance,
        end_distance,
        if wrap {
            ContourSliceMode::Wrap
        } else {
            ContourSliceMode::Clamp
        },
        Tolerance::default(),
    )
    .map_err(js_err)?;
    Ok(serialize_path(&sliced, SerializeOptions::default()))
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
    let segments_a: Vec<_> = pa
        .subpaths()
        .iter()
        .flat_map(|subpath| subpath.segments().iter().copied())
        .collect();
    let segments_b: Vec<_> = pb
        .subpaths()
        .iter()
        .flat_map(|subpath| subpath.segments().iter().copied())
        .collect();
    let bounds_a: Vec<_> = segments_a.iter().map(|segment| segment.bounds()).collect();
    let bounds_b: Vec<_> = segments_b.iter().map(|segment| segment.bounds()).collect();
    let candidates =
        spatial_cross_candidate_pairs(&bounds_a, &bounds_b, tol.absolute).map_err(js_err)?;
    let mut points = Vec::new();

    for (index_a, index_b) in candidates {
        let r = match intersect_segments(segments_a[index_a], segments_b[index_b], tol) {
            Ok(v) => v,
            Err(pfx_vector_core::CoreError::UnsupportedCase) => continue,
            Err(e) => return Err(js_err(e)),
        };
        for hit in r.intersections {
            if let pfx_vector_core::Intersection::Point(p) = hit {
                points.push(p.point);
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

fn parse_stroke_join(value: &str) -> Result<StrokeJoin, JsValue> {
    match value {
        "miter" => Ok(StrokeJoin::Miter),
        "round" => Ok(StrokeJoin::Round),
        "bevel" => Ok(StrokeJoin::Bevel),
        _ => Err(JsValue::from_str("invalid stroke join")),
    }
}

fn parse_stroke_cap(value: &str) -> Result<StrokeCap, JsValue> {
    match value {
        "butt" => Ok(StrokeCap::Butt),
        "round" => Ok(StrokeCap::Round),
        "square" => Ok(StrokeCap::Square),
        _ => Err(JsValue::from_str("invalid stroke cap")),
    }
}

fn parse_dash_array(value: &str) -> Result<Vec<f64>, JsValue> {
    if value.trim().is_empty() {
        return Ok(Vec::new());
    }

    value
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<f64>()
                .map_err(|_| JsValue::from_str("invalid dash array"))
        })
        .collect()
}

#[wasm_bindgen]
pub fn dash_path_svg(data: &str, dash_array: &str, dash_offset: f64) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let style = StrokeStyle {
        dash_array: parse_dash_array(dash_array)?,
        dash_offset,
        ..StrokeStyle::default()
    };
    let result = dash_path(&path, &style, Tolerance::default()).map_err(js_err)?;
    Ok(serialize_path(&result, SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn offset_path_svg(
    data: &str,
    distance: f64,
    join: &str,
    miter_limit: f64,
) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let result = offset_path(
        &path,
        distance,
        OffsetStyle {
            join: parse_stroke_join(join)?,
            miter_limit,
        },
        Tolerance::default(),
    )
    .map_err(js_err)?;

    Ok(serialize_path(&result, SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn outline_path_svg(
    data: &str,
    width: f64,
    cap: &str,
    join: &str,
    miter_limit: f64,
) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let style = StrokeStyle {
        width,
        cap: parse_stroke_cap(cap)?,
        join: parse_stroke_join(join)?,
        miter_limit,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };
    let result = outline_path(&path, &style, Tolerance::default()).map_err(js_err)?;

    Ok(serialize_path(&result, SerializeOptions::default()))
}

fn mesh_json(mesh: &Mesh2D) -> String {
    let vertices = mesh
        .vertices
        .iter()
        .flat_map(|point| [point.x.to_string(), point.y.to_string()])
        .collect::<Vec<_>>()
        .join(",");
    let indices = mesh
        .indices
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");

    format!("{{\"vertices\":[{vertices}],\"indices\":[{indices}]}}")
}

#[wasm_bindgen]
pub fn outline_dashed_path_svg(
    data: &str,
    width: f64,
    cap: &str,
    join: &str,
    miter_limit: f64,
    dash_array: &str,
    dash_offset: f64,
) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let style = StrokeStyle {
        width,
        cap: parse_stroke_cap(cap)?,
        join: parse_stroke_join(join)?,
        miter_limit,
        dash_array: parse_dash_array(dash_array)?,
        dash_offset,
    };
    let result = outline_path(&path, &style, Tolerance::default()).map_err(js_err)?;
    Ok(serialize_path(&result, SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn tessellate_fill_svg(data: &str, even_odd: bool, flatness: f64) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let tolerance = Tolerance {
        flatness: flatness.max(1.0e-12),
        ..Tolerance::default()
    };
    let mesh = tessellate_fill(
        &path,
        if even_odd {
            FillRule::EvenOdd
        } else {
            FillRule::NonZero
        },
        tolerance,
    )
    .map_err(js_err)?;

    Ok(mesh_json(&mesh))
}

#[wasm_bindgen]
pub fn tessellate_stroke_svg(
    data: &str,
    width: f64,
    cap: &str,
    join: &str,
    miter_limit: f64,
    flatness: f64,
) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let tolerance = Tolerance {
        flatness: flatness.max(1.0e-12),
        ..Tolerance::default()
    };
    let style = StrokeStyle {
        width,
        cap: parse_stroke_cap(cap)?,
        join: parse_stroke_join(join)?,
        miter_limit,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };
    let mesh = tessellate_stroke(&path, &style, tolerance).map_err(js_err)?;

    Ok(mesh_json(&mesh))
}

#[wasm_bindgen]
pub fn cleanup_path_svg(
    data: &str,
    point_tolerance: f64,
    collinear_tolerance: f64,
) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let cleaned = cleanup_path(
        &path,
        CleanupOptions {
            point_tolerance,
            collinear_tolerance,
        },
        Tolerance::default(),
    )
    .map_err(js_err)?;

    Ok(serialize_path(&cleaned, SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn simplify_path_svg(data: &str, max_deviation: f64) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let simplified = simplify_path(&path, max_deviation, Tolerance::default()).map_err(js_err)?;

    Ok(serialize_path(&simplified, SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn fit_path_curves_svg(data: &str, max_error: f64) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let fitted = fit_path_curves(&path, max_error, Tolerance::default()).map_err(js_err)?;

    Ok(serialize_path(&fitted, SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn normalize_fill_contours_svg(data: &str, even_odd: bool) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let normalized = normalize_self_intersections(
        &path,
        if even_odd {
            FillRule::EvenOdd
        } else {
            FillRule::NonZero
        },
        Tolerance::default(),
    )
    .map_err(js_err)?;

    Ok(serialize_path(&normalized, SerializeOptions::default()))
}
