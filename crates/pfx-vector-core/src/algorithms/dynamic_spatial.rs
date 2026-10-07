use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

use crate::{Bounds, CoreError, CoreResult, Point2, Scalar, SpatialIndex};

/// Mutable broad-phase index for geometry sets whose bounds change over time.
///
/// The index keeps a deterministic immutable sweep index as its base and a
/// bounded overlay for inserts, updates, and removals. Once enough distinct
/// items have changed, the base is rebuilt automatically. Queries always see
/// the latest geometry, including changes that have not yet triggered a
/// rebuild.
#[derive(Clone, Debug, Default)]
pub struct DynamicSpatialIndex {
    items: BTreeMap<usize, Bounds>,
    base_ids: Vec<usize>,
    base: SpatialIndex,
    dirty: BTreeSet<usize>,
}

impl DynamicSpatialIndex {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bounds(bounds: &[Bounds]) -> CoreResult<Self> {
        let mut items = BTreeMap::new();
        for (item_index, &item_bounds) in bounds.iter().enumerate() {
            if item_bounds == Bounds::Empty {
                continue;
            }
            validate_indexed_bounds(item_bounds)?;
            items.insert(item_index, item_bounds);
        }

        let mut index = Self {
            items,
            base_ids: Vec::new(),
            base: SpatialIndex::default(),
            dirty: BTreeSet::new(),
        };
        index.rebuild();
        Ok(index)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[must_use]
    pub fn contains_item(&self, item_index: usize) -> bool {
        self.items.contains_key(&item_index)
    }

    #[must_use]
    pub fn item_bounds(&self, item_index: usize) -> Option<Bounds> {
        self.items.get(&item_index).copied()
    }

    pub fn insert(&mut self, item_index: usize, bounds: Bounds) -> CoreResult<()> {
        validate_indexed_bounds(bounds)?;

        match self.items.entry(item_index) {
            Entry::Vacant(entry) => {
                entry.insert(bounds);
            }
            Entry::Occupied(_) => return Err(CoreError::InvalidGeometry),
        }

        self.dirty.insert(item_index);
        self.maybe_rebuild();
        Ok(())
    }

    pub fn update(&mut self, item_index: usize, bounds: Bounds) -> CoreResult<()> {
        validate_indexed_bounds(bounds)?;
        let Some(current) = self.items.get_mut(&item_index) else {
            return Err(CoreError::InvalidGeometry);
        };

        if *current == bounds {
            return Ok(());
        }

        *current = bounds;
        self.dirty.insert(item_index);
        self.maybe_rebuild();
        Ok(())
    }

    pub fn remove(&mut self, item_index: usize) -> CoreResult<Bounds> {
        let bounds = self
            .items
            .remove(&item_index)
            .ok_or(CoreError::InvalidGeometry)?;
        self.dirty.insert(item_index);
        self.maybe_rebuild();
        Ok(bounds)
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.base_ids.clear();
        self.base = SpatialIndex::default();
        self.dirty.clear();
    }

    /// Forces pending mutations into a fresh deterministic sweep index.
    pub fn rebuild(&mut self) {
        self.base_ids.clear();
        self.base_ids.reserve(self.items.len());
        let mut bounds = Vec::with_capacity(self.items.len());

        for (&item_index, &item_bounds) in &self.items {
            self.base_ids.push(item_index);
            bounds.push(item_bounds);
        }

        self.base = SpatialIndex::new(&bounds);
        self.dirty.clear();
    }

    pub fn query_bounds(&self, query: Bounds, padding: Scalar) -> CoreResult<Vec<usize>> {
        validate_padding(padding)?;
        if query == Bounds::Empty {
            return Ok(Vec::new());
        }

        let base_matches = self.base.query_bounds(query, padding)?;
        let mut result = Vec::with_capacity(base_matches.len() + self.dirty.len());

        for base_index in base_matches {
            let item_index = self.base_ids[base_index];
            if !self.dirty.contains(&item_index) {
                result.push(item_index);
            }
        }

        for item_index in &self.dirty {
            let Some(&item_bounds) = self.items.get(item_index) else {
                continue;
            };
            if bounds_overlap(item_bounds, query, padding) {
                result.push(*item_index);
            }
        }

        result.sort_unstable();
        result.dedup();
        Ok(result)
    }

    pub fn self_candidate_pairs(&self, padding: Scalar) -> CoreResult<Vec<(usize, usize)>> {
        validate_padding(padding)?;
        let mut pairs = Vec::new();

        for (&left_index, &left_bounds) in &self.items {
            for right_index in self.query_bounds(left_bounds, padding)? {
                if right_index > left_index {
                    pairs.push((left_index, right_index));
                }
            }
        }

        pairs.sort_unstable();
        pairs.dedup();
        Ok(pairs)
    }

    pub fn cross_candidate_pairs(
        &self,
        other: &Self,
        padding: Scalar,
    ) -> CoreResult<Vec<(usize, usize)>> {
        validate_padding(padding)?;
        let mut pairs = Vec::new();

        for (&left_index, &left_bounds) in &self.items {
            for right_index in other.query_bounds(left_bounds, padding)? {
                pairs.push((left_index, right_index));
            }
        }

        pairs.sort_unstable();
        pairs.dedup();
        Ok(pairs)
    }

    fn maybe_rebuild(&mut self) {
        let scale = self.items.len().max(self.base_ids.len());
        let threshold = (scale / 16).clamp(32, 1024);
        if self.dirty.len() >= threshold {
            self.rebuild();
        }
    }
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

fn validate_indexed_bounds(bounds: Bounds) -> CoreResult<()> {
    let Bounds::Finite { min, max } = bounds else {
        return Err(CoreError::InvalidGeometry);
    };

    if !point_is_finite(min) || !point_is_finite(max) {
        return Err(CoreError::InvalidNumber);
    }
    if min.x > max.x || min.y > max.y {
        return Err(CoreError::InvalidGeometry);
    }

    Ok(())
}

fn point_is_finite(point: Point2) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

fn bounds_overlap(left: Bounds, right: Bounds, padding: Scalar) -> bool {
    left.expanded(padding).intersects(right)
}
