use crate::{
    CoreError, CoreResult, FillRule, Path, Point2, Scalar, StrokeStyle, Tolerance, flatten_path,
    normalize_self_intersections, outline_path,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh2D {
    pub vertices: Vec<Point2>,
    pub indices: Vec<u32>,
}

impl Mesh2D {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

#[derive(Clone, Debug)]
struct BoundaryContour {
    points: Vec<Point2>,
    area: Scalar,
}

pub fn tessellate_fill(
    path: &Path,
    fill_rule: FillRule,
    tolerance: Tolerance,
) -> CoreResult<Mesh2D> {
    if path.subpaths().iter().any(|subpath| !subpath.is_closed()) {
        return Err(CoreError::UnsupportedCase);
    }

    let normalized = normalize_self_intersections(path, fill_rule, tolerance)?;
    let mut contours = Vec::new();
    for subpath in flatten_path(&normalized, tolerance)? {
        let points = sanitize_contour(subpath.points, tolerance);
        if points.len() < 3 {
            continue;
        }

        let area = signed_area(&points);
        if area.abs() > area_epsilon(&points, tolerance) {
            contours.push(BoundaryContour { points, area });
        }
    }

    let mut outers: Vec<BoundaryContour> = contours
        .iter()
        .filter(|contour| contour.area > 0.0)
        .cloned()
        .collect();
    let mut holes: Vec<BoundaryContour> = contours
        .into_iter()
        .filter(|contour| contour.area < 0.0)
        .collect();

    outers.sort_by(|left, right| right.area.total_cmp(&left.area));
    holes.sort_by(|left, right| rightmost_x(&right.points).total_cmp(&rightmost_x(&left.points)));

    let mut grouped: Vec<(Vec<Point2>, Vec<Vec<Point2>>)> = outers
        .into_iter()
        .map(|outer| (outer.points, Vec::new()))
        .collect();

    for hole in holes {
        let sample = hole.points[0];
        let mut best: Option<(usize, Scalar)> = None;
        for (index, (outer, _)) in grouped.iter().enumerate() {
            if point_in_polygon(sample, outer, tolerance) {
                let area = signed_area(outer).abs();
                if best.is_none_or(|(_, best_area)| area < best_area) {
                    best = Some((index, area));
                }
            }
        }

        let Some((index, _)) = best else {
            return Err(CoreError::InvalidGeometry);
        };
        grouped[index].1.push(hole.points);
    }

    let mut mesh = Mesh2D::default();
    for (outer, mut component_holes) in grouped {
        component_holes.sort_by(|left, right| rightmost_x(right).total_cmp(&rightmost_x(left)));
        let mut polygon = outer;
        for hole in component_holes {
            polygon = bridge_hole(polygon, hole, tolerance)?;
        }
        triangulate_simple_polygon(&polygon, tolerance, &mut mesh)?;
    }

    Ok(mesh)
}

pub fn tessellate_stroke(
    path: &Path,
    style: &StrokeStyle,
    tolerance: Tolerance,
) -> CoreResult<Mesh2D> {
    let outline = outline_path(path, style, tolerance)?;
    tessellate_fill(&outline, FillRule::NonZero, tolerance)
}

fn sanitize_contour(mut points: Vec<Point2>, tolerance: Tolerance) -> Vec<Point2> {
    if points.len() > 1 && points[0].almost_eq(*points.last().unwrap(), tolerance) {
        points.pop();
    }

    let mut cleaned = Vec::with_capacity(points.len());
    for point in points {
        if cleaned
            .last()
            .is_none_or(|previous: &Point2| !previous.almost_eq(point, tolerance))
        {
            cleaned.push(point);
        }
    }

    loop {
        if cleaned.len() < 3 {
            break;
        }

        let mut removed = false;
        for index in 0..cleaned.len() {
            let a = cleaned[(index + cleaned.len() - 1) % cleaned.len()];
            let b = cleaned[index];
            let c = cleaned[(index + 1) % cleaned.len()];
            let cross = (b - a).cross(c - b).abs();
            let scale = a.distance_to(b).max(b.distance_to(c)).max(1.0);
            if cross <= (tolerance.absolute + tolerance.relative * scale) * scale {
                cleaned.remove(index);
                removed = true;
                break;
            }
        }

        if !removed {
            break;
        }
    }

    cleaned
}

fn signed_area(points: &[Point2]) -> Scalar {
    if points.len() < 3 {
        return 0.0;
    }

    let origin = points[0];
    let mut sum = 0.0;
    for index in 0..points.len() {
        let a = points[index] - origin;
        let b = points[(index + 1) % points.len()] - origin;
        sum += a.cross(b);
    }
    sum * 0.5
}

fn area_epsilon(points: &[Point2], tolerance: Tolerance) -> Scalar {
    if points.is_empty() {
        return tolerance.absolute;
    }

    let mut min_x = points[0].x;
    let mut min_y = points[0].y;
    let mut max_x = points[0].x;
    let mut max_y = points[0].y;

    for point in &points[1..] {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }

    let scale = (max_x - min_x).max(max_y - min_y).max(1.0);
    (tolerance.absolute + tolerance.relative * scale) * scale
}

fn rightmost_x(points: &[Point2]) -> Scalar {
    points
        .iter()
        .map(|point| point.x)
        .fold(Scalar::NEG_INFINITY, Scalar::max)
}

fn point_in_polygon(point: Point2, polygon: &[Point2], tolerance: Tolerance) -> bool {
    let mut inside = false;
    for index in 0..polygon.len() {
        let a = polygon[index];
        let b = polygon[(index + 1) % polygon.len()];
        let ab = b - a;
        let ap = point - a;
        if ab.cross(ap).abs()
            <= tolerance
                .absolute
                .max(tolerance.relative * ab.length() * ap.length())
            && point.x >= a.x.min(b.x) - tolerance.absolute
            && point.x <= a.x.max(b.x) + tolerance.absolute
            && point.y >= a.y.min(b.y) - tolerance.absolute
            && point.y <= a.y.max(b.y) + tolerance.absolute
        {
            return true;
        }

        if (a.y > point.y) != (b.y > point.y) {
            let x = a.x + (point.y - a.y) * (b.x - a.x) / (b.y - a.y);
            if x > point.x {
                inside = !inside;
            }
        }
    }
    inside
}

fn bridge_hole(
    mut polygon: Vec<Point2>,
    hole: Vec<Point2>,
    tolerance: Tolerance,
) -> CoreResult<Vec<Point2>> {
    let hole_index = hole
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| {
            left.x
                .total_cmp(&right.x)
                .then_with(|| right.y.total_cmp(&left.y))
        })
        .map(|(index, _)| index)
        .ok_or(CoreError::InvalidGeometry)?;
    let hole_point = hole[hole_index];

    let mut best: Option<(usize, Point2, Scalar)> = None;
    for index in 0..polygon.len() {
        let a = polygon[index];
        let b = polygon[(index + 1) % polygon.len()];
        if (a.y > hole_point.y) == (b.y > hole_point.y) {
            continue;
        }

        let x = a.x + (hole_point.y - a.y) * (b.x - a.x) / (b.y - a.y);
        if x <= hole_point.x + tolerance.absolute {
            continue;
        }

        if best.is_none_or(|(_, _, best_x)| x < best_x) {
            best = Some((index, Point2::new(x, hole_point.y), x));
        }
    }

    let Some((edge_index, bridge_point, _)) = best else {
        return Err(CoreError::NonConvergent);
    };

    let next_index = (edge_index + 1) % polygon.len();
    let bridge_index = if bridge_point.almost_eq(polygon[edge_index], tolerance) {
        edge_index
    } else if bridge_point.almost_eq(polygon[next_index], tolerance) {
        next_index
    } else if next_index == 0 {
        polygon.push(bridge_point);
        polygon.len() - 1
    } else {
        polygon.insert(next_index, bridge_point);
        next_index
    };

    let mut merged = Vec::with_capacity(polygon.len() + hole.len() + 2);
    merged.extend_from_slice(&polygon[..=bridge_index]);
    for offset in 0..hole.len() {
        merged.push(hole[(hole_index + offset) % hole.len()]);
    }
    merged.push(hole[hole_index]);
    merged.push(polygon[bridge_index]);
    merged.extend_from_slice(&polygon[bridge_index + 1..]);

    Ok(merged)
}

fn triangulate_simple_polygon(
    polygon: &[Point2],
    tolerance: Tolerance,
    mesh: &mut Mesh2D,
) -> CoreResult<()> {
    if polygon.len() < 3 {
        return Ok(());
    }

    let mut points = polygon.to_vec();
    if signed_area(&points) < 0.0 {
        points.reverse();
    }

    let base = u32::try_from(mesh.vertices.len()).map_err(|_| CoreError::IterationLimit)?;
    let point_count = u32::try_from(points.len()).map_err(|_| CoreError::IterationLimit)?;
    if base.checked_add(point_count).is_none() {
        return Err(CoreError::IterationLimit);
    }

    let mut remaining: Vec<usize> = (0..points.len()).collect();
    let epsilon = area_epsilon(&points, tolerance);
    let mut local_indices = Vec::with_capacity((points.len() - 2) * 3);
    let mut guard = points.len().saturating_mul(points.len()).saturating_mul(4);

    while remaining.len() > 3 {
        if guard == 0 {
            return Err(CoreError::NonConvergent);
        }
        guard -= 1;

        let mut ear_found = false;
        for position in 0..remaining.len() {
            let prev = remaining[(position + remaining.len() - 1) % remaining.len()];
            let current = remaining[position];
            let next = remaining[(position + 1) % remaining.len()];
            let a = points[prev];
            let b = points[current];
            let c = points[next];

            if (b - a).cross(c - b) <= epsilon {
                continue;
            }

            let blocked = remaining.iter().copied().any(|candidate| {
                if candidate == prev || candidate == current || candidate == next {
                    return false;
                }
                let p = points[candidate];
                if p.almost_eq(a, tolerance)
                    || p.almost_eq(b, tolerance)
                    || p.almost_eq(c, tolerance)
                {
                    return false;
                }
                point_inside_or_on_triangle(p, a, b, c, epsilon)
            });

            if blocked {
                continue;
            }

            local_indices.extend_from_slice(&[
                u32::try_from(prev).unwrap(),
                u32::try_from(current).unwrap(),
                u32::try_from(next).unwrap(),
            ]);
            remaining.remove(position);
            ear_found = true;
            break;
        }

        if !ear_found {
            let mut removed = false;
            for position in 0..remaining.len() {
                let prev = remaining[(position + remaining.len() - 1) % remaining.len()];
                let current = remaining[position];
                let next = remaining[(position + 1) % remaining.len()];
                if (points[current] - points[prev])
                    .cross(points[next] - points[current])
                    .abs()
                    <= epsilon
                {
                    remaining.remove(position);
                    removed = true;
                    break;
                }
            }

            if !removed {
                return Err(CoreError::NonConvergent);
            }
        }
    }

    if remaining.len() == 3 {
        let a = points[remaining[0]];
        let b = points[remaining[1]];
        let c = points[remaining[2]];
        if (b - a).cross(c - b) > epsilon {
            local_indices.extend_from_slice(&[
                u32::try_from(remaining[0]).unwrap(),
                u32::try_from(remaining[1]).unwrap(),
                u32::try_from(remaining[2]).unwrap(),
            ]);
        }
    }

    mesh.vertices.extend(points);
    mesh.indices
        .extend(local_indices.into_iter().map(|index| base + index));
    Ok(())
}

fn point_inside_or_on_triangle(
    point: Point2,
    a: Point2,
    b: Point2,
    c: Point2,
    epsilon: Scalar,
) -> bool {
    let ab = (b - a).cross(point - a);
    let bc = (c - b).cross(point - b);
    let ca = (a - c).cross(point - c);
    ab >= -epsilon && bc >= -epsilon && ca >= -epsilon
}
