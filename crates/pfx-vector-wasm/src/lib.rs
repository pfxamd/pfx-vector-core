#![forbid(unsafe_code)]
use pfx_vector_core::{
    BooleanOperation, Bounds, CleanupOptions, ContourSliceMode, FillRule, Mesh2D, OffsetStyle,
    PathBuilder, Point2, Segment, SegmentAddress, StrokeCap, StrokeJoin, StrokeStyle, Subpath,
    SubpathEndpoint, Tolerance, Transform2D, boolean_paths, cleanup_path, closest_point,
    contains_point, contour_length, dash_path, extract_segment, extract_subpath, fit_path_curves,
    flatten_path,
    intersect_segments, join_open_subpaths, normalize_self_intersections, offset_path,
    outline_path, path_length, point_at_length, remove_segment, remove_subpath, replace_segment,
    replace_subpath, reverse_subpath, set_subpath_closed, simplify_path, slice_contour,
    spatial_cross_candidate_pairs, split_segment, split_segment_at_length, stroke_contains_point,
    tessellate_fill, tessellate_stroke, transform_path,
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

fn append_edit_segment(builder: &mut PathBuilder, segment: Segment) -> Result<(), JsValue> {
    match segment {
        Segment::Line(line) => {
            builder.line_to(line.end).map_err(js_err)?;
        }
        Segment::Quadratic(curve) => {
            builder.quad_to(curve.p1, curve.p2).map_err(js_err)?;
        }
        Segment::Cubic(curve) => {
            builder
                .cubic_to(curve.p1, curve.p2, curve.p3)
                .map_err(js_err)?;
        }
        Segment::Arc(arc) => {
            builder.arc_to(arc).map_err(js_err)?;
        }
    }

    Ok(())
}

fn serialize_edit_segment(segment: Segment) -> Result<String, JsValue> {
    let mut builder = PathBuilder::new();
    builder.move_to(segment.start()).map_err(js_err)?;
    append_edit_segment(&mut builder, segment)?;
    let path = builder.finish().map_err(js_err)?;
    Ok(serialize_path(&path, SerializeOptions::default()))
}

fn serialize_edit_subpath(subpath: &Subpath) -> Result<String, JsValue> {
    let mut builder = PathBuilder::new();
    builder.move_to(subpath.start()).map_err(js_err)?;

    for &segment in subpath.segments() {
        append_edit_segment(&mut builder, segment)?;
    }

    if subpath.is_closed() {
        builder.close().map_err(js_err)?;
    }

    let path = builder.finish().map_err(js_err)?;
    Ok(serialize_path(&path, SerializeOptions::default()))
}

fn parse_single_replacement_segment(data: &str) -> Result<Segment, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    if path.subpaths().len() != 1 || path.subpaths()[0].segments().len() != 1 {
        return Err(JsValue::from_str(
            "replacement must contain exactly one subpath and one segment",
        ));
    }

    Ok(path.subpaths()[0].segments()[0])
}

fn parse_single_replacement_subpath(data: &str) -> Result<Subpath, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    if path.subpaths().len() != 1 {
        return Err(JsValue::from_str(
            "replacement must contain exactly one subpath",
        ));
    }

    Ok(path.subpaths()[0].clone())
}

#[wasm_bindgen]
pub fn split_segment_svg(
    data: &str,
    subpath_index: u32,
    segment_index: u32,
    t: f64,
) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let edit = split_segment(
        &path,
        SegmentAddress::new(subpath_index as usize, segment_index as usize),
        t,
        Tolerance::default(),
    )
    .map_err(js_err)?;
    Ok(serialize_path(edit.path(), SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn split_segment_at_length_svg(
    data: &str,
    subpath_index: u32,
    segment_index: u32,
    distance: f64,
) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let edit = split_segment_at_length(
        &path,
        SegmentAddress::new(subpath_index as usize, segment_index as usize),
        distance,
        Tolerance::default(),
    )
    .map_err(js_err)?;
    Ok(serialize_path(edit.path(), SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn reverse_subpath_svg(data: &str, subpath_index: u32) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let edit = reverse_subpath(&path, subpath_index as usize).map_err(js_err)?;
    Ok(serialize_path(edit.path(), SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn set_subpath_closed_svg(
    data: &str,
    subpath_index: u32,
    closed: bool,
) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let edit = set_subpath_closed(&path, subpath_index as usize, closed, Tolerance::default())
        .map_err(js_err)?;
    Ok(serialize_path(edit.path(), SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn extract_segment_svg(
    data: &str,
    subpath_index: u32,
    segment_index: u32,
) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let segment = extract_segment(
        &path,
        SegmentAddress::new(subpath_index as usize, segment_index as usize),
    )
    .map_err(js_err)?;
    serialize_edit_segment(segment)
}

#[wasm_bindgen]
pub fn extract_subpath_svg(data: &str, subpath_index: u32) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let subpath = extract_subpath(&path, subpath_index as usize).map_err(js_err)?;
    serialize_edit_subpath(&subpath)
}

#[wasm_bindgen]
pub fn replace_segment_svg(
    data: &str,
    subpath_index: u32,
    segment_index: u32,
    replacement_data: &str,
) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let replacement = parse_single_replacement_segment(replacement_data)?;
    let edit = replace_segment(
        &path,
        SegmentAddress::new(subpath_index as usize, segment_index as usize),
        replacement,
        Tolerance::default(),
    )
    .map_err(js_err)?;
    Ok(serialize_path(edit.path(), SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn replace_subpath_svg(
    data: &str,
    subpath_index: u32,
    replacement_data: &str,
) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let replacement = parse_single_replacement_subpath(replacement_data)?;
    let edit = replace_subpath(&path, subpath_index as usize, replacement).map_err(js_err)?;
    Ok(serialize_path(edit.path(), SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn remove_segment_svg(
    data: &str,
    subpath_index: u32,
    segment_index: u32,
) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let edit = remove_segment(
        &path,
        SegmentAddress::new(subpath_index as usize, segment_index as usize),
        Tolerance::default(),
    )
    .map_err(js_err)?;
    Ok(serialize_path(edit.path(), SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn remove_subpath_svg(data: &str, subpath_index: u32) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let edit = remove_subpath(&path, subpath_index as usize).map_err(js_err)?;
    Ok(serialize_path(edit.path(), SerializeOptions::default()))
}

#[wasm_bindgen]
pub fn join_subpaths_svg(
    data: &str,
    first_index: u32,
    first_at_start: bool,
    second_index: u32,
    second_at_start: bool,
) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let edit = join_open_subpaths(
        &path,
        first_index as usize,
        if first_at_start {
            SubpathEndpoint::Start
        } else {
            SubpathEndpoint::End
        },
        second_index as usize,
        if second_at_start {
            SubpathEndpoint::Start
        } else {
            SubpathEndpoint::End
        },
        Tolerance::default(),
    )
    .map_err(js_err)?;
    Ok(serialize_path(edit.path(), SerializeOptions::default()))
}

fn parse_transform_matrix(value: &str) -> Result<Transform2D, JsValue> {
    let values = value
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<f64>()
                .map_err(|_| JsValue::from_str("invalid transform matrix"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if values.len() != 6 {
        return Err(JsValue::from_str("transform matrix must contain 6 values"));
    }

    Ok(Transform2D::new(
        values[0], values[1], values[2], values[3], values[4], values[5],
    ))
}

#[wasm_bindgen]
pub fn transform_path_svg(data: &str, matrix: &str, flatness: f64) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let tolerance = Tolerance {
        flatness: flatness.max(1.0e-12),
        ..Tolerance::default()
    };
    let transformed =
        transform_path(&path, parse_transform_matrix(matrix)?, tolerance).map_err(js_err)?;
    Ok(serialize_path(&transformed, SerializeOptions::default()))
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
pub fn closest_point_svg(data: &str, x: f64, y: f64) -> Result<String, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let result =
        closest_point(&path, Point2::new(x, y), Tolerance::default()).map_err(js_err)?;
    Ok(format!(
        "{{\"x\":{},\"y\":{},\"distance\":{},\"distanceSquared\":{},\"subpath\":{},\"segment\":{},\"t\":{},\"pathDistance\":{}}}",
        result.point.x,
        result.point.y,
        result.distance,
        result.distance_squared,
        result.location.subpath_index,
        result.location.segment_index,
        result.location.t,
        result.location.distance
    ))
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
pub fn hit_test_stroke_svg(
    data: &str,
    x: f64,
    y: f64,
    width: f64,
    cap: &str,
    join: &str,
    miter_limit: f64,
) -> Result<bool, JsValue> {
    let path = parse_path(data).map_err(js_err)?;
    let style = StrokeStyle {
        width,
        cap: parse_stroke_cap(cap)?,
        join: parse_stroke_join(join)?,
        miter_limit,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };

    stroke_contains_point(&path, &style, Point2::new(x, y), Tolerance::default()).map_err(js_err)
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
