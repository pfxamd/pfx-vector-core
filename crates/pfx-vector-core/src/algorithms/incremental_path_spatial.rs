use std::collections::BTreeMap;

use super::flatten::{FlattenedSegmentEdge, flatten_segment_edges};
use super::measure::segment_length_to_t;
use crate::{
    Bounds, CoreError, CoreResult, DynamicSpatialIndex, FillRule, Path, PathLocation, Point2,
    PointClassification, Scalar, Segment, SegmentClosestPoint, StrokeStyle, Subpath, Tolerance,
    closest_point_on_segment, segment_length,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PathSpatialSync {
    pub full_rebuild: bool,
    pub changed_subpaths: usize,
    pub changed_segments: usize,
    pub inserted_edges: usize,
    pub removed_edges: usize,
}

#[derive(Clone, Copy, Debug)]
struct IncrementalPathEdge;

#[derive(Clone, Copy, Debug)]
struct PreparedEdge {
    start: Point2,
    end: Point2,
}

#[derive(Clone, Debug)]
enum PreparedCloseUpdate {
    Keep,
    Set(Option<PreparedEdge>),
}

#[derive(Clone, Debug)]
struct PreparedSubpathChange {
    subpath_index: usize,
    replace_all: bool,
    segment_updates: Vec<(usize, Vec<PreparedEdge>)>,
    close_update: PreparedCloseUpdate,
    full_lengths: Option<Vec<Scalar>>,
    length_updates: Vec<(usize, Scalar)>,
    bounds: Bounds,
    changed_segments: usize,
}

#[derive(Clone, Debug)]
pub struct IncrementalPathSpatialIndex {
    snapshot: Path,
    tolerance: Tolerance,
    edges: BTreeMap<usize, IncrementalPathEdge>,
    spatial: DynamicSpatialIndex,
    segment_spatial: DynamicSpatialIndex,
    segment_sources: BTreeMap<usize, (usize, usize)>,
    segment_source_ids: Vec<Vec<usize>>,
    next_segment_id: usize,
    segment_edge_ids: BTreeMap<(usize, usize), Vec<usize>>,
    closing_edge_ids: BTreeMap<usize, usize>,
    next_edge_id: usize,
    subpath_bounds: Vec<Bounds>,
    segment_lengths: Vec<Vec<Scalar>>,
    segment_offsets: Vec<Vec<Scalar>>,
    subpath_offsets: Vec<Scalar>,
    bounds: Bounds,
    total_length: Scalar,
}

impl IncrementalPathSpatialIndex {
    pub fn build(path: &Path, tolerance: Tolerance) -> CoreResult<Self> {
        let mut index = Self {
            snapshot: path.clone(),
            tolerance,
            edges: BTreeMap::new(),
            spatial: DynamicSpatialIndex::new(),
            segment_spatial: DynamicSpatialIndex::new(),
            segment_sources: BTreeMap::new(),
            segment_source_ids: Vec::with_capacity(path.subpaths().len()),
            next_segment_id: 0,
            segment_edge_ids: BTreeMap::new(),
            closing_edge_ids: BTreeMap::new(),
            next_edge_id: 0,
            subpath_bounds: Vec::with_capacity(path.subpaths().len()),
            segment_lengths: Vec::with_capacity(path.subpaths().len()),
            segment_offsets: Vec::new(),
            subpath_offsets: Vec::new(),
            bounds: Bounds::Empty,
            total_length: 0.0,
        };

        for (subpath_index, subpath) in path.subpaths().iter().enumerate() {
            index
                .segment_source_ids
                .push(Vec::with_capacity(subpath.segments().len()));
            let prepared = prepare_all_segments(subpath, tolerance)?;
            index.ensure_edge_capacity(prepared_edge_count(&prepared))?;
            for (segment_index, edges) in prepared.into_iter().enumerate() {
                index.insert_segment_edges(subpath_index, segment_index, &edges)?;
            }
            for (segment_index, &segment) in subpath.segments().iter().enumerate() {
                index.insert_segment_bound(subpath_index, segment_index, segment)?;
            }

            if let Some(closing) = prepare_closing_edge(subpath) {
                index.ensure_edge_capacity(1)?;
                index.insert_closing_edge(subpath_index, closing)?;
            }

            index.subpath_bounds.push(subpath_bounds(subpath));
            index
                .segment_lengths
                .push(compute_segment_lengths(subpath, tolerance)?);
        }

        index.recompute_metrics();
        index.recompute_bounds();
        Ok(index)
    }

    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.snapshot.segment_count()
    }

    #[must_use]
    pub fn bounds(&self) -> Bounds {
        self.bounds
    }

    #[must_use]
    pub fn total_length(&self) -> Scalar {
        self.total_length
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.snapshot
    }

    pub fn sync_edit(&mut self, edit: &crate::PathEditResult) -> CoreResult<PathSpatialSync> {
        self.sync_path(edit.path())
    }

    pub fn sync_path(&mut self, path: &Path) -> CoreResult<PathSpatialSync> {
        if self.snapshot == *path {
            return Ok(PathSpatialSync::default());
        }

        if self.snapshot.subpaths().len() != path.subpaths().len() {
            let previous_edges = self.edge_count();
            let changed_subpaths = self.snapshot.subpaths().len().max(path.subpaths().len());
            let changed_segments = self.snapshot.segment_count().max(path.segment_count());
            let rebuilt = Self::build(path, self.tolerance)?;
            let inserted_edges = rebuilt.edge_count();
            *self = rebuilt;
            return Ok(PathSpatialSync {
                full_rebuild: true,
                changed_subpaths,
                changed_segments,
                inserted_edges,
                removed_edges: previous_edges,
            });
        }

        let mut prepared_changes = Vec::new();
        let old_subpaths = self.snapshot.subpaths();

        for (subpath_index, new_subpath) in path.subpaths().iter().enumerate() {
            let old_subpath = &old_subpaths[subpath_index];
            if old_subpath == new_subpath {
                continue;
            }

            prepared_changes.push(prepare_subpath_change(
                subpath_index,
                old_subpath,
                new_subpath,
                self.tolerance,
            )?);
        }

        let requested_new_edges: usize = prepared_changes
            .iter()
            .map(prepared_change_edge_count)
            .sum();
        self.ensure_edge_capacity(requested_new_edges)?;

        let mut report = PathSpatialSync::default();

        for change in prepared_changes {
            let new_subpath = &path.subpaths()[change.subpath_index];
            report.changed_subpaths += 1;
            report.changed_segments += change.changed_segments;

            if change.replace_all {
                self.remove_subpath_segment_bounds(change.subpath_index)?;
                for segment_index in 0..self.snapshot.subpaths()[change.subpath_index]
                    .segments()
                    .len()
                {
                    report.removed_edges +=
                        self.remove_segment_edges(change.subpath_index, segment_index)?;
                }
                report.removed_edges += self.remove_closing_edge(change.subpath_index)?;

                for (segment_index, edges) in &change.segment_updates {
                    report.inserted_edges +=
                        self.insert_segment_edges(change.subpath_index, *segment_index, edges)?;
                }
                for (segment_index, &segment) in new_subpath.segments().iter().enumerate() {
                    self.insert_segment_bound(change.subpath_index, segment_index, segment)?;
                }

                if let PreparedCloseUpdate::Set(Some(edge)) = change.close_update {
                    self.insert_closing_edge(change.subpath_index, edge)?;
                    report.inserted_edges += 1;
                }

                self.segment_lengths[change.subpath_index] =
                    change.full_lengths.ok_or(CoreError::InvalidGeometry)?;
            } else {
                for (segment_index, edges) in &change.segment_updates {
                    report.removed_edges +=
                        self.remove_segment_edges(change.subpath_index, *segment_index)?;
                    report.inserted_edges +=
                        self.insert_segment_edges(change.subpath_index, *segment_index, edges)?;
                    self.update_segment_bound(
                        change.subpath_index,
                        *segment_index,
                        new_subpath.segments()[*segment_index],
                    )?;
                }

                match change.close_update {
                    PreparedCloseUpdate::Keep => {}
                    PreparedCloseUpdate::Set(edge) => {
                        report.removed_edges += self.remove_closing_edge(change.subpath_index)?;
                        if let Some(edge) = edge {
                            self.insert_closing_edge(change.subpath_index, edge)?;
                            report.inserted_edges += 1;
                        }
                    }
                }

                for (segment_index, length) in change.length_updates {
                    self.segment_lengths[change.subpath_index][segment_index] = length;
                }
            }

            self.subpath_bounds[change.subpath_index] = change.bounds;
        }

        self.snapshot = path.clone();
        self.recompute_metrics();
        self.recompute_bounds();
        Ok(report)
    }

    pub fn classify_point(&self, point: Point2, rule: FillRule) -> CoreResult<PointClassification> {
        if !point.is_finite() {
            return Err(CoreError::InvalidNumber);
        }
        if self.segment_sources.is_empty() && self.edges.is_empty() {
            return Ok(PointClassification::Outside);
        }

        let Bounds::Finite { max, .. } = self.bounds else {
            return Ok(PointClassification::Outside);
        };

        let numerical = point_tolerance(point, self.tolerance);
        if point.x > max.x + numerical {
            return Ok(PointClassification::Outside);
        }

        let ray_y = point.y + numerical * 0.25;
        let query = Bounds::Finite {
            min: Point2::new(point.x - numerical, point.y.min(ray_y) - numerical),
            max: Point2::new(max.x + numerical, point.y.max(ray_y) + numerical),
        };
        let source_candidates = self.segment_spatial.query_bounds(query, 0.0)?;
        let edge_candidates = self.spatial.query_bounds(query, 0.0)?;
        if source_candidates.is_empty() && edge_candidates.is_empty() {
            return Ok(PointClassification::Outside);
        }

        crate::classify_point(&self.snapshot, point, rule, self.tolerance)
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

        crate::stroke_contains_point(&self.snapshot, style, point, self.tolerance)
    }

    pub fn closest_point(&self, point: Point2) -> CoreResult<crate::ClosestPointResult> {
        if !point.is_finite() {
            return Err(CoreError::InvalidNumber);
        }
        if self.segment_sources.is_empty() {
            return Err(CoreError::DegenerateOperation);
        }

        let Bounds::Finite { min, max } = self.bounds else {
            return Err(CoreError::DegenerateOperation);
        };
        let scale = (max.x - min.x)
            .max(max.y - min.y)
            .max(self.tolerance.absolute)
            .max(1.0e-12);
        let minimum = self.bounds.distance_squared_to_point(point).sqrt();
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
                    && (candidate.1.0, candidate.1.1, candidate.0.t)
                        < (best.1.0, best.1.1, best.0.t))
            {
                best = candidate;
            }
        }

        let (candidate, (subpath_index, segment_index)) = best;
        let distance = self.distance_to_location(subpath_index, segment_index, candidate.t)?;

        Ok(crate::ClosestPointResult {
            point: candidate.point,
            distance: candidate.distance,
            distance_squared: candidate.distance_squared,
            location: PathLocation {
                subpath_index,
                segment_index,
                t: candidate.t,
                distance: distance.min(self.total_length),
            },
        })
    }

    fn closest_from_segment_candidates(
        &self,
        candidates: &[usize],
        point: Point2,
    ) -> CoreResult<Option<(SegmentClosestPoint, (usize, usize))>> {
        let mut best = None;

        for &segment_id in candidates {
            let source = *self
                .segment_sources
                .get(&segment_id)
                .ok_or(CoreError::InvalidGeometry)?;
            let segment = self.snapshot.subpaths()[source.0].segments()[source.1];
            let candidate = closest_point_on_segment(segment, point, self.tolerance)?;

            if best.is_none_or(
                |(current, current_source): (SegmentClosestPoint, (usize, usize))| {
                    candidate.distance_squared < current.distance_squared
                        || (candidate.distance_squared == current.distance_squared
                            && (source.0, source.1, candidate.t)
                                < (current_source.0, current_source.1, current.t))
                },
            ) {
                best = Some((candidate, source));
            }
        }

        Ok(best)
    }

    fn distance_to_location(
        &self,
        subpath_index: usize,
        segment_index: usize,
        t: Scalar,
    ) -> CoreResult<Scalar> {
        let segment = self.snapshot.subpaths()[subpath_index].segments()[segment_index];
        Ok(self.subpath_offsets[subpath_index]
            + self.segment_offsets[subpath_index][segment_index]
            + segment_length_to_t(segment, t, self.tolerance)?)
    }

    fn insert_segment_bound(
        &mut self,
        subpath_index: usize,
        segment_index: usize,
        segment: Segment,
    ) -> CoreResult<()> {
        let next = self
            .next_segment_id
            .checked_add(1)
            .ok_or(CoreError::IterationLimit)?;
        let segment_id = self.next_segment_id;
        self.segment_spatial.insert(segment_id, segment.bounds())?;
        self.segment_sources
            .insert(segment_id, (subpath_index, segment_index));
        self.segment_source_ids[subpath_index].push(segment_id);
        self.next_segment_id = next;
        Ok(())
    }

    fn update_segment_bound(
        &mut self,
        subpath_index: usize,
        segment_index: usize,
        segment: Segment,
    ) -> CoreResult<()> {
        let segment_id = *self
            .segment_source_ids
            .get(subpath_index)
            .and_then(|ids| ids.get(segment_index))
            .ok_or(CoreError::InvalidGeometry)?;
        self.segment_spatial.update(segment_id, segment.bounds())
    }

    fn remove_subpath_segment_bounds(&mut self, subpath_index: usize) -> CoreResult<()> {
        let ids = self
            .segment_source_ids
            .get_mut(subpath_index)
            .ok_or(CoreError::InvalidGeometry)?;
        let removed = core::mem::take(ids);
        for segment_id in removed {
            self.segment_spatial.remove(segment_id)?;
            self.segment_sources
                .remove(&segment_id)
                .ok_or(CoreError::InvalidGeometry)?;
        }
        Ok(())
    }

    fn insert_segment_edges(
        &mut self,
        subpath_index: usize,
        segment_index: usize,
        prepared: &[PreparedEdge],
    ) -> CoreResult<usize> {
        let mut ids = Vec::with_capacity(prepared.len());

        for &edge in prepared {
            let edge_id = self.insert_edge(subpath_index, edge)?;
            ids.push(edge_id);
        }

        self.segment_edge_ids
            .insert((subpath_index, segment_index), ids);
        Ok(prepared.len())
    }

    fn insert_closing_edge(
        &mut self,
        subpath_index: usize,
        prepared: PreparedEdge,
    ) -> CoreResult<()> {
        let edge_id = self.insert_edge(subpath_index, prepared)?;
        self.closing_edge_ids.insert(subpath_index, edge_id);
        Ok(())
    }

    fn insert_edge(&mut self, _subpath_index: usize, prepared: PreparedEdge) -> CoreResult<usize> {
        let next = self
            .next_edge_id
            .checked_add(1)
            .ok_or(CoreError::IterationLimit)?;
        let edge_id = self.next_edge_id;
        let bounds = Bounds::from_points(&[prepared.start, prepared.end]);
        self.spatial.insert(edge_id, bounds)?;
        self.edges.insert(edge_id, IncrementalPathEdge);
        self.next_edge_id = next;
        Ok(edge_id)
    }

    fn remove_segment_edges(
        &mut self,
        subpath_index: usize,
        segment_index: usize,
    ) -> CoreResult<usize> {
        let ids = self
            .segment_edge_ids
            .remove(&(subpath_index, segment_index))
            .ok_or(CoreError::InvalidGeometry)?;

        for edge_id in &ids {
            self.remove_edge(*edge_id)?;
        }

        Ok(ids.len())
    }

    fn remove_closing_edge(&mut self, subpath_index: usize) -> CoreResult<usize> {
        let Some(edge_id) = self.closing_edge_ids.remove(&subpath_index) else {
            return Ok(0);
        };
        self.remove_edge(edge_id)?;
        Ok(1)
    }

    fn remove_edge(&mut self, edge_id: usize) -> CoreResult<()> {
        self.spatial.remove(edge_id)?;
        self.edges
            .remove(&edge_id)
            .ok_or(CoreError::InvalidGeometry)?;
        Ok(())
    }

    fn ensure_edge_capacity(&self, count: usize) -> CoreResult<()> {
        self.next_edge_id
            .checked_add(count)
            .ok_or(CoreError::IterationLimit)?;
        Ok(())
    }

    fn recompute_metrics(&mut self) {
        self.segment_offsets.clear();
        self.segment_offsets.reserve(self.segment_lengths.len());
        self.subpath_offsets.clear();
        self.subpath_offsets.reserve(self.segment_lengths.len());

        let mut path_offset = 0.0;

        for lengths in &self.segment_lengths {
            self.subpath_offsets.push(path_offset);
            let mut segment_offset = 0.0;
            let mut offsets = Vec::with_capacity(lengths.len());

            for &length in lengths {
                offsets.push(segment_offset);
                segment_offset += length;
            }

            path_offset += segment_offset;
            self.segment_offsets.push(offsets);
        }

        self.total_length = path_offset;
    }

    fn recompute_bounds(&mut self) {
        self.bounds = self
            .subpath_bounds
            .iter()
            .copied()
            .fold(Bounds::Empty, Bounds::union);
    }
}

fn prepare_subpath_change(
    subpath_index: usize,
    old_subpath: &Subpath,
    new_subpath: &Subpath,
    tolerance: Tolerance,
) -> CoreResult<PreparedSubpathChange> {
    let replace_all = old_subpath.start() != new_subpath.start()
        || old_subpath.is_closed() != new_subpath.is_closed()
        || old_subpath.segments().len() != new_subpath.segments().len();

    if replace_all {
        let prepared = prepare_all_segments(new_subpath, tolerance)?;
        let changed_segments = old_subpath
            .segments()
            .len()
            .max(new_subpath.segments().len());
        return Ok(PreparedSubpathChange {
            subpath_index,
            replace_all: true,
            segment_updates: prepared.into_iter().enumerate().collect(),
            close_update: PreparedCloseUpdate::Set(prepare_closing_edge(new_subpath)),
            full_lengths: Some(compute_segment_lengths(new_subpath, tolerance)?),
            length_updates: Vec::new(),
            bounds: subpath_bounds(new_subpath),
            changed_segments,
        });
    }

    let mut segment_updates = Vec::new();
    let mut length_updates = Vec::new();

    for (segment_index, (&old_segment, &new_segment)) in old_subpath
        .segments()
        .iter()
        .zip(new_subpath.segments())
        .enumerate()
    {
        if old_segment == new_segment {
            continue;
        }

        segment_updates.push((
            segment_index,
            prepare_segment(new_segment, segment_index, tolerance)?,
        ));
        length_updates.push((segment_index, segment_length(new_segment, tolerance)?));
    }

    let close_update = if old_subpath.end() == new_subpath.end() {
        PreparedCloseUpdate::Keep
    } else {
        PreparedCloseUpdate::Set(prepare_closing_edge(new_subpath))
    };

    Ok(PreparedSubpathChange {
        subpath_index,
        replace_all: false,
        changed_segments: segment_updates.len(),
        segment_updates,
        close_update,
        full_lengths: None,
        length_updates,
        bounds: subpath_bounds(new_subpath),
    })
}

fn prepare_all_segments(
    subpath: &Subpath,
    tolerance: Tolerance,
) -> CoreResult<Vec<Vec<PreparedEdge>>> {
    subpath
        .segments()
        .iter()
        .copied()
        .enumerate()
        .map(|(segment_index, segment)| prepare_segment(segment, segment_index, tolerance))
        .collect()
}

fn prepare_segment(
    segment: Segment,
    _segment_index: usize,
    tolerance: Tolerance,
) -> CoreResult<Vec<PreparedEdge>> {
    flatten_segment_edges(segment, tolerance)?
        .into_iter()
        .map(prepared_source_edge)
        .collect()
}

fn prepared_source_edge(edge: FlattenedSegmentEdge) -> CoreResult<PreparedEdge> {
    validate_edge_points(edge.start, edge.end)?;
    Ok(PreparedEdge {
        start: edge.start,
        end: edge.end,
    })
}

fn prepare_closing_edge(subpath: &Subpath) -> Option<PreparedEdge> {
    if !subpath.is_closed() || subpath.segments().is_empty() {
        return None;
    }

    let start = subpath.start();
    let end = subpath.end();
    if start == end {
        return None;
    }

    Some(PreparedEdge {
        start: end,
        end: start,
    })
}

fn prepared_change_edge_count(change: &PreparedSubpathChange) -> usize {
    change
        .segment_updates
        .iter()
        .map(|(_, edges)| edges.len())
        .sum::<usize>()
        + match change.close_update {
            PreparedCloseUpdate::Set(Some(_)) => 1,
            PreparedCloseUpdate::Keep | PreparedCloseUpdate::Set(None) => 0,
        }
}

fn prepared_edge_count(segments: &[Vec<PreparedEdge>]) -> usize {
    segments.iter().map(Vec::len).sum()
}

fn compute_segment_lengths(subpath: &Subpath, tolerance: Tolerance) -> CoreResult<Vec<Scalar>> {
    subpath
        .segments()
        .iter()
        .copied()
        .map(|segment| segment_length(segment, tolerance))
        .collect()
}

fn subpath_bounds(subpath: &Subpath) -> Bounds {
    let mut bounds = subpath
        .segments()
        .iter()
        .copied()
        .map(Segment::bounds)
        .fold(Bounds::Empty, Bounds::union);

    if subpath.is_closed() && !subpath.segments().is_empty() && subpath.start() != subpath.end() {
        bounds = bounds.union(Bounds::from_points(&[subpath.end(), subpath.start()]));
    }

    bounds
}

fn validate_edge_points(start: Point2, end: Point2) -> CoreResult<()> {
    if !start.is_finite() || !end.is_finite() {
        return Err(CoreError::InvalidNumber);
    }
    Ok(())
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
