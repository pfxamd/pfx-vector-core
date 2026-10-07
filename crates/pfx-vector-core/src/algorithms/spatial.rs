use crate::{
    Bounds, CoreError, CoreResult, FillRule, Path, PathLocation, Point2, PointClassification,
    Scalar, StrokeStyle, Tolerance, flatten_path,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpatialEntry {
    pub item_index: usize,
    pub bounds: Bounds,
}

#[derive(Clone, Debug, Default)]
pub struct SpatialIndex {
    entries: Vec<SpatialEntry>,
    prefix_max_x: Vec<Scalar>,
    bounds: Bounds,
}

impl SpatialIndex {
    #[must_use]
    pub fn new(bounds: &[Bounds]) -> Self {
        let mut entries: Vec<_> = bounds
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(item_index, bounds)| match bounds {
                Bounds::Empty => None,
                Bounds::Finite { .. } => Some(SpatialEntry { item_index, bounds }),
            })
            .collect();

        entries.sort_by(|left, right| {
            bounds_min_x(left.bounds)
                .total_cmp(&bounds_min_x(right.bounds))
                .then_with(|| left.item_index.cmp(&right.item_index))
        });

        let mut prefix_max_x = Vec::with_capacity(entries.len());
        let mut maximum = Scalar::NEG_INFINITY;
        let mut total = Bounds::Empty;

        for entry in &entries {
            maximum = maximum.max(bounds_max_x(entry.bounds));
            prefix_max_x.push(maximum);
            total = total.union(entry.bounds);
        }

        Self {
            entries,
            prefix_max_x,
            bounds: total,
        }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub const fn bounds(&self) -> Bounds {
        self.bounds
    }

    pub fn query_bounds(&self, query: Bounds, padding: Scalar) -> CoreResult<Vec<usize>> {
        validate_padding(padding)?;
        let Some((query_min, query_max)) = finite_bounds(query) else {
            return Ok(Vec::new());
        };

        let min_x = query_min.x - padding;
        let max_x = query_max.x + padding;
        let end = self
            .entries
            .partition_point(|entry| bounds_min_x(entry.bounds) <= max_x);
        let start = self.prefix_max_x[..end].partition_point(|maximum| *maximum < min_x);

        let mut result = Vec::new();
        for entry in &self.entries[start..end] {
            if bounds_overlap(entry.bounds, query, padding) {
                result.push(entry.item_index);
            }
        }

        result.sort_unstable();
        Ok(result)
    }
}

pub fn spatial_self_candidate_pairs(
    bounds: &[Bounds],
    padding: Scalar,
) -> CoreResult<Vec<(usize, usize)>> {
    validate_padding(padding)?;
    let index = SpatialIndex::new(bounds);
    let mut pairs = Vec::new();

    for (left_index, &left_bounds) in bounds.iter().enumerate() {
        for right_index in index.query_bounds(left_bounds, padding)? {
            if right_index > left_index {
                pairs.push((left_index, right_index));
            }
        }
    }

    pairs.sort_unstable();
    pairs.dedup();
    Ok(pairs)
}

pub fn spatial_cross_candidate_pairs(
    left: &[Bounds],
    right: &[Bounds],
    padding: Scalar,
) -> CoreResult<Vec<(usize, usize)>> {
    validate_padding(padding)?;
    let index = SpatialIndex::new(right);
    let mut pairs = Vec::new();

    for (left_index, &left_bounds) in left.iter().enumerate() {
        for right_index in index.query_bounds(left_bounds, padding)? {
            pairs.push((left_index, right_index));
        }
    }

    pairs.sort_unstable();
    pairs.dedup();
    Ok(pairs)
}

#[derive(Clone, Copy, Debug)]
struct IndexedEdge {
    start: Point2,
    end: Point2,
    subpath_index: usize,
    segment_index: usize,
    distance_start: Scalar,
}

#[derive(Clone, Debug)]
pub struct PathSpatialIndex {
    edges: Vec<IndexedEdge>,
    spatial: SpatialIndex,
    tolerance: Tolerance,
    total_length: Scalar,
}

impl PathSpatialIndex {
    pub fn build(path: &Path, tolerance: Tolerance) -> CoreResult<Self> {
        let flattened = flatten_path(path, tolerance)?;
        let mut edges = Vec::new();
        let mut bounds = Vec::new();
        let mut distance = 0.0;

        for (subpath_index, subpath) in flattened.iter().enumerate() {
            for (segment_index, window) in subpath.points.windows(2).enumerate() {
                push_edge(
                    &mut edges,
                    &mut bounds,
                    window[0],
                    window[1],
                    subpath_index,
                    segment_index,
                    &mut distance,
                );
            }

            if subpath.closed && subpath.points.len() > 2 {
                let start = subpath.points[0];
                let end = *subpath.points.last().expect("closed subpath is non-empty");
                if start != end {
                    push_edge(
                        &mut edges,
                        &mut bounds,
                        end,
                        start,
                        subpath_index,
                        subpath.points.len() - 1,
                        &mut distance,
                    );
                }
            }
        }

        Ok(Self {
            spatial: SpatialIndex::new(&bounds),
            edges,
            tolerance,
            total_length: distance,
        })
    }

    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    #[must_use]
    pub const fn bounds(&self) -> Bounds {
        self.spatial.bounds()
    }

    pub fn classify_point(
        &self,
        point: Point2,
        rule: FillRule,
    ) -> CoreResult<PointClassification> {
        if self.edges.is_empty() {
            return Ok(PointClassification::Outside);
        }

        let Bounds::Finite { max, .. } = self.spatial.bounds() else {
            return Ok(PointClassification::Outside);
        };

        let numerical = point_tolerance(point, self.tolerance);
        if point.x > max.x + numerical {
            return Ok(PointClassification::Outside);
        }

        let query = Bounds::Finite {
            min: Point2::new(point.x - numerical, point.y - numerical),
            max: Point2::new(max.x + numerical, point.y + numerical),
        };
        let candidates = self.spatial.query_bounds(query, numerical)?;
        let mut winding = 0i32;
        let mut parity = false;

        for index in candidates {
            let edge = self.edges[index];
            let a = edge.start;
            let b = edge.end;
            let ab = b - a;
            let ap = point - a;
            let cross = ab.cross(ap).abs();

            if cross
                <= self
                    .tolerance
                    .absolute
                    .max(self.tolerance.relative * ab.length() * ap.length())
                && point.x >= a.x.min(b.x) - self.tolerance.absolute
                && point.x <= a.x.max(b.x) + self.tolerance.absolute
                && point.y >= a.y.min(b.y) - self.tolerance.absolute
                && point.y <= a.y.max(b.y) + self.tolerance.absolute
            {
                return Ok(PointClassification::Boundary);
            }

            let crosses = (a.y > point.y) != (b.y > point.y);
            if crosses {
                let x = a.x + (point.y - a.y) * (b.x - a.x) / (b.y - a.y);
                if x > point.x {
                    parity = !parity;
                    if b.y > a.y {
                        winding += 1;
                    } else {
                        winding -= 1;
                    }
                }
            }
        }

        Ok(match rule {
            FillRule::EvenOdd if parity => PointClassification::Inside,
            FillRule::NonZero if winding != 0 => PointClassification::Inside,
            _ => PointClassification::Outside,
        })
    }

    pub fn contains_point(&self, point: Point2, rule: FillRule) -> CoreResult<bool> {
        Ok(self.classify_point(point, rule)? != PointClassification::Outside)
    }

    pub fn stroke_contains_point(&self, style: &StrokeStyle, point: Point2) -> CoreResult<bool> {
        style.validate()?;
        if style.width == 0.0 || self.edges.is_empty() {
            return Ok(false);
        }

        let radius = style.width * 0.5 + self.tolerance.absolute;
        let query = square_bounds(point, radius);
        for index in self.spatial.query_bounds(query, 0.0)? {
            let edge = self.edges[index];
            if distance_to_segment(point, edge.start, edge.end) <= radius {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn closest_point(&self, point: Point2) -> CoreResult<crate::ClosestPointResult> {
        if self.edges.is_empty() {
            return Err(CoreError::DegenerateOperation);
        }

        let Bounds::Finite { min, max } = self.spatial.bounds() else {
            return Err(CoreError::DegenerateOperation);
        };
        let scale = (max.x - min.x)
            .max(max.y - min.y)
            .max(self.tolerance.absolute)
            .max(1.0e-12);
        let minimum = bounds_distance_squared_to_point(self.spatial.bounds(), point).sqrt();
        let mut radius = (scale / 64.0).max(minimum + self.tolerance.absolute);
        let mut candidates = Vec::new();

        for _ in 0..64 {
            candidates = self.spatial.query_bounds(square_bounds(point, radius), 0.0)?;
            if !candidates.is_empty() {
                break;
            }
            radius *= 2.0;
        }

        if candidates.is_empty() {
            return Err(CoreError::NonConvergent);
        }

        let mut best = closest_from_candidates(&self.edges, &candidates, point)
            .ok_or(CoreError::DegenerateOperation)?;
        let final_radius = best.0.sqrt() + self.tolerance.absolute;
        candidates = self
            .spatial
            .query_bounds(square_bounds(point, final_radius), 0.0)?;

        if let Some(candidate) = closest_from_candidates(&self.edges, &candidates, point) {
            if candidate.0 < best.0 {
                best = candidate;
            }
        }

        let (distance_squared, candidate, edge, t) = best;
        let edge_length = edge.start.distance_to(edge.end);

        Ok(crate::ClosestPointResult {
            point: candidate,
            distance: distance_squared.sqrt(),
            distance_squared,
            location: PathLocation {
                subpath_index: edge.subpath_index,
                segment_index: edge.segment_index,
                t,
                distance: (edge.distance_start + t * edge_length).min(self.total_length),
            },
        })
    }
}

fn push_edge(
    edges: &mut Vec<IndexedEdge>,
    bounds: &mut Vec<Bounds>,
    start: Point2,
    end: Point2,
    subpath_index: usize,
    segment_index: usize,
    distance: &mut Scalar,
) {
    let edge = IndexedEdge {
        start,
        end,
        subpath_index,
        segment_index,
        distance_start: *distance,
    };
    edges.push(edge);
    bounds.push(Bounds::from_points(&[start, end]));
    *distance += start.distance_to(end);
}

fn closest_from_candidates(
    edges: &[IndexedEdge],
    candidates: &[usize],
    point: Point2,
) -> Option<(Scalar, Point2, IndexedEdge, Scalar)> {
    let mut best: Option<(Scalar, Point2, IndexedEdge, Scalar)> = None;

    for &index in candidates {
        let edge = edges[index];
        let (candidate, t) = closest_line(edge.start, edge.end, point);
        let distance_squared = candidate.distance_squared_to(point);

        if best.is_none_or(|value| distance_squared < value.0) {
            best = Some((distance_squared, candidate, edge, t));
        }
    }

    best
}

fn closest_line(start: Point2, end: Point2, point: Point2) -> (Point2, Scalar) {
    let direction = end - start;
    let length_squared = direction.length_squared();

    if length_squared == 0.0 {
        return (start, 0.0);
    }

    let t = ((point - start).dot(direction) / length_squared).clamp(0.0, 1.0);
    (start + direction * t, t)
}

fn distance_to_segment(point: Point2, start: Point2, end: Point2) -> Scalar {
    closest_line(start, end, point).0.distance_to(point)
}

fn square_bounds(center: Point2, radius: Scalar) -> Bounds {
    Bounds::Finite {
        min: Point2::new(center.x - radius, center.y - radius),
        max: Point2::new(center.x + radius, center.y + radius),
    }
}

fn point_tolerance(point: Point2, tolerance: Tolerance) -> Scalar {
    tolerance.absolute
        + tolerance.relative * point.x.abs().max(point.y.abs()).max(1.0)
}

fn validate_padding(padding: Scalar) -> CoreResult<()> {
    if !padding.is_finite() {
        return Err(CoreError::InvalidNumber);
    }
    if padding < 0.0 {
        return Err(CoreError::InvalidGeometry);
    }
    Ok(())
}

fn bounds_overlap(left: Bounds, right: Bounds, padding: Scalar) -> bool {
    let Some((left_min, left_max)) = finite_bounds(left) else {
        return false;
    };
    let Some((right_min, right_max)) = finite_bounds(right) else {
        return false;
    };

    left_min.x <= right_max.x + padding
        && left_max.x + padding >= right_min.x
        && left_min.y <= right_max.y + padding
        && left_max.y + padding >= right_min.y
}

fn bounds_distance_squared_to_point(bounds: Bounds, point: Point2) -> Scalar {
    let Some((min, max)) = finite_bounds(bounds) else {
        return Scalar::INFINITY;
    };

    let dx = if point.x < min.x {
        min.x - point.x
    } else if point.x > max.x {
        point.x - max.x
    } else {
        0.0
    };
    let dy = if point.y < min.y {
        min.y - point.y
    } else if point.y > max.y {
        point.y - max.y
    } else {
        0.0
    };

    dx * dx + dy * dy
}

fn finite_bounds(bounds: Bounds) -> Option<(Point2, Point2)> {
    match bounds {
        Bounds::Empty => None,
        Bounds::Finite { min, max } => Some((min, max)),
    }
}

fn bounds_min_x(bounds: Bounds) -> Scalar {
    finite_bounds(bounds).map_or(Scalar::INFINITY, |(min, _)| min.x)
}

fn bounds_max_x(bounds: Bounds) -> Scalar {
    finite_bounds(bounds).map_or(Scalar::NEG_INFINITY, |(_, max)| max.x)
}
