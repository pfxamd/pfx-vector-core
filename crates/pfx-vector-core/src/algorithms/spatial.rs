use super::measure::segment_length_to_t;
use crate::{
    Bounds, CoreError, CoreResult, FillRule, Path, PathLocation, Point2, PointClassification,
    Scalar, SegmentClosestPoint, StrokeStyle, Tolerance, closest_point_on_segment, flatten_path,
    segment_length,
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
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
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
}

#[derive(Clone, Copy, Debug)]
struct IndexedSegment {
    subpath_index: usize,
    segment_index: usize,
    distance_start: Scalar,
}

#[derive(Clone, Debug)]
pub struct PathSpatialIndex {
    path: Path,
    edges: Vec<IndexedEdge>,
    spatial: SpatialIndex,
    segments: Vec<IndexedSegment>,
    segment_spatial: SpatialIndex,
    tolerance: Tolerance,
    total_length: Scalar,
}

impl PathSpatialIndex {
    pub fn build(path: &Path, tolerance: Tolerance) -> CoreResult<Self> {
        let flattened = flatten_path(path, tolerance)?;
        let mut edges = Vec::new();
        let mut bounds = Vec::new();

        for (subpath_index, subpath) in flattened.iter().enumerate() {
            for (segment_index, window) in subpath.points.windows(2).enumerate() {
                push_edge(
                    &mut edges,
                    &mut bounds,
                    window[0],
                    window[1],
                    subpath_index,
                    segment_index,
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
                    );
                }
            }
        }

        let mut segments = Vec::with_capacity(path.segment_count());
        let mut segment_bounds = Vec::with_capacity(path.segment_count());
        let mut total_length = 0.0;

        for (subpath_index, subpath) in path.subpaths().iter().enumerate() {
            for (segment_index, &segment) in subpath.segments().iter().enumerate() {
                segments.push(IndexedSegment {
                    subpath_index,
                    segment_index,
                    distance_start: total_length,
                });
                segment_bounds.push(segment.bounds());
                total_length += segment_length(segment, tolerance)?;
            }
        }

        Ok(Self {
            path: path.clone(),
            spatial: SpatialIndex::new(&bounds),
            edges,
            segments,
            segment_spatial: SpatialIndex::new(&segment_bounds),
            tolerance,
            total_length,
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

    pub fn classify_point(&self, point: Point2, rule: FillRule) -> CoreResult<PointClassification> {
        if !point.is_finite() {
            return Err(CoreError::InvalidNumber);
        }
        if self.segments.is_empty() && self.edges.is_empty() {
            return Ok(PointClassification::Outside);
        }

        let bounds = self.segment_spatial.bounds().union(self.spatial.bounds());
        let Bounds::Finite { max, .. } = bounds else {
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
        let source_candidates = self.segment_spatial.query_bounds(query, numerical)?;
        let flattened_candidates = self.spatial.query_bounds(query, numerical)?;

        if source_candidates.is_empty() && flattened_candidates.is_empty() {
            return Ok(PointClassification::Outside);
        }

        crate::classify_point(&self.path, point, rule, self.tolerance)
    }

    pub fn contains_point(&self, point: Point2, rule: FillRule) -> CoreResult<bool> {
        Ok(self.classify_point(point, rule)? != PointClassification::Outside)
    }

    pub fn stroke_contains_point(&self, style: &StrokeStyle, point: Point2) -> CoreResult<bool> {
        style.validate()?;
        if style.width == 0.0 || self.edges.is_empty() {
            return Ok(false);
        }

        let radius = crate::algorithms::stroke::stroke_query_radius(style, self.tolerance);
        if self
            .spatial
            .query_bounds(square_bounds(point, radius), 0.0)?
            .is_empty()
        {
            return Ok(false);
        }

        crate::stroke_contains_point(&self.path, style, point, self.tolerance)
    }

    pub fn closest_point(&self, point: Point2) -> CoreResult<crate::ClosestPointResult> {
        if !point.is_finite() {
            return Err(CoreError::InvalidNumber);
        }
        if self.segments.is_empty() {
            return Err(CoreError::DegenerateOperation);
        }

        let Bounds::Finite { min, max } = self.segment_spatial.bounds() else {
            return Err(CoreError::DegenerateOperation);
        };
        let scale = (max.x - min.x)
            .max(max.y - min.y)
            .max(self.tolerance.absolute)
            .max(1.0e-12);
        let minimum = self
            .segment_spatial
            .bounds()
            .distance_squared_to_point(point)
            .sqrt();
        let mut radius = (scale / 64.0).max(minimum + self.tolerance.absolute);
        let mut candidates = Vec::new();

        for _ in 0..64 {
            candidates = self
                .segment_spatial
                .query_bounds(square_bounds(point, radius), 0.0)?;
            if !candidates.is_empty() {
                break;
            }
            radius *= 2.0;
        }

        if candidates.is_empty() {
            return Err(CoreError::NonConvergent);
        }

        let mut best = self
            .closest_from_segment_candidates(&candidates, point)?
            .ok_or(CoreError::DegenerateOperation)?;
        let final_radius = best.0.distance + self.tolerance.absolute;
        candidates = self
            .segment_spatial
            .query_bounds(square_bounds(point, final_radius), 0.0)?;

        if let Some(candidate) = self.closest_from_segment_candidates(&candidates, point)? {
            if candidate.0.distance_squared < best.0.distance_squared
                || (candidate.0.distance_squared == best.0.distance_squared
                    && (
                        candidate.1.subpath_index,
                        candidate.1.segment_index,
                        candidate.0.t,
                    ) < (best.1.subpath_index, best.1.segment_index, best.0.t))
            {
                best = candidate;
            }
        }

        let (candidate, source) = best;
        let segment = self.path.subpaths()[source.subpath_index].segments()[source.segment_index];
        let distance =
            source.distance_start + segment_length_to_t(segment, candidate.t, self.tolerance)?;

        Ok(crate::ClosestPointResult {
            point: candidate.point,
            distance: candidate.distance,
            distance_squared: candidate.distance_squared,
            location: PathLocation {
                subpath_index: source.subpath_index,
                segment_index: source.segment_index,
                t: candidate.t,
                distance: distance.min(self.total_length),
            },
        })
    }

    fn closest_from_segment_candidates(
        &self,
        candidates: &[usize],
        point: Point2,
    ) -> CoreResult<Option<(SegmentClosestPoint, IndexedSegment)>> {
        let mut best = None;

        for &index in candidates {
            let source = self.segments[index];
            let segment =
                self.path.subpaths()[source.subpath_index].segments()[source.segment_index];
            let candidate = closest_point_on_segment(segment, point, self.tolerance)?;

            if best.is_none_or(
                |(current, current_source): (SegmentClosestPoint, IndexedSegment)| {
                    candidate.distance_squared < current.distance_squared
                        || (candidate.distance_squared == current.distance_squared
                            && (source.subpath_index, source.segment_index, candidate.t)
                                < (
                                    current_source.subpath_index,
                                    current_source.segment_index,
                                    current.t,
                                ))
                },
            ) {
                best = Some((candidate, source));
            }
        }

        Ok(best)
    }
}

fn push_edge(
    edges: &mut Vec<IndexedEdge>,
    bounds: &mut Vec<Bounds>,
    start: Point2,
    end: Point2,
    _subpath_index: usize,
    _segment_index: usize,
) {
    edges.push(IndexedEdge { start, end });
    bounds.push(Bounds::from_points(&[start, end]));
}

fn square_bounds(center: Point2, radius: Scalar) -> Bounds {
    Bounds::Finite {
        min: Point2::new(center.x - radius, center.y - radius),
        max: Point2::new(center.x + radius, center.y + radius),
    }
}

fn point_tolerance(point: Point2, tolerance: Tolerance) -> Scalar {
    tolerance.absolute + tolerance.relative * point.x.abs().max(point.y.abs()).max(1.0)
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
