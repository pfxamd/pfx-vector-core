use std::collections::BTreeMap;

use pfx_vector_core::*;

fn bounds(x0: f64, y0: f64, x1: f64, y1: f64) -> Bounds {
    Bounds::Finite {
        min: Point2::new(x0, y0),
        max: Point2::new(x1, y1),
    }
}

fn brute_query(items: &BTreeMap<usize, Bounds>, query: Bounds, padding: f64) -> Vec<usize> {
    items
        .iter()
        .filter_map(|(&item_index, &item_bounds)| {
            item_bounds
                .expanded(padding)
                .intersects(query)
                .then_some(item_index)
        })
        .collect()
}

fn brute_self_pairs(items: &BTreeMap<usize, Bounds>, padding: f64) -> Vec<(usize, usize)> {
    let entries: Vec<_> = items
        .iter()
        .map(|(&index, &bounds)| (index, bounds))
        .collect();
    let mut pairs = Vec::new();

    for left in 0..entries.len() {
        for right in left + 1..entries.len() {
            if entries[left]
                .1
                .expanded(padding)
                .intersects(entries[right].1)
            {
                pairs.push((entries[left].0, entries[right].0));
            }
        }
    }

    pairs
}

fn brute_cross_pairs(
    left: &BTreeMap<usize, Bounds>,
    right: &BTreeMap<usize, Bounds>,
    padding: f64,
) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();

    for (&left_index, &left_bounds) in left {
        for (&right_index, &right_bounds) in right {
            if left_bounds.expanded(padding).intersects(right_bounds) {
                pairs.push((left_index, right_index));
            }
        }
    }

    pairs
}

#[test]
fn dynamic_index_insert_update_remove_and_stable_queries() {
    let mut index = DynamicSpatialIndex::new();

    index.insert(7, bounds(0.0, 0.0, 4.0, 4.0)).unwrap();
    index.insert(2, bounds(3.0, 2.0, 8.0, 6.0)).unwrap();
    index.insert(19, bounds(30.0, 30.0, 31.0, 31.0)).unwrap();

    let query = bounds(1.0, 1.0, 5.0, 5.0);
    assert_eq!(index.query_bounds(query, 0.0).unwrap(), vec![2, 7]);
    assert_eq!(index.query_bounds(query, 0.0).unwrap(), vec![2, 7]);
    assert_eq!(index.len(), 3);
    assert!(index.contains_item(19));
    assert_eq!(index.item_bounds(7), Some(bounds(0.0, 0.0, 4.0, 4.0)));

    index.update(7, bounds(50.0, 50.0, 55.0, 55.0)).unwrap();
    assert_eq!(index.query_bounds(query, 0.0).unwrap(), vec![2]);

    assert_eq!(index.remove(2).unwrap(), bounds(3.0, 2.0, 8.0, 6.0));
    assert!(index.query_bounds(query, 0.0).unwrap().is_empty());
    assert!(!index.contains_item(2));
}

#[test]
fn dynamic_index_from_bounds_preserves_original_item_indices() {
    let source = vec![
        bounds(0.0, 0.0, 2.0, 2.0),
        Bounds::Empty,
        bounds(1.0, 1.0, 3.0, 3.0),
        bounds(20.0, 20.0, 21.0, 21.0),
    ];
    let index = DynamicSpatialIndex::from_bounds(&source).unwrap();

    assert_eq!(index.len(), 3);
    assert_eq!(
        index.query_bounds(bounds(0.5, 0.5, 2.5, 2.5), 0.0).unwrap(),
        vec![0, 2]
    );
}

#[test]
fn dynamic_index_matches_brute_force_across_many_mutations() {
    let mut index = DynamicSpatialIndex::new();
    let mut items = BTreeMap::new();

    for item_index in 0..160 {
        let x = item_index as f64 * 3.25;
        let item_bounds = bounds(x, -2.0, x + 1.5, 2.0);
        index.insert(item_index, item_bounds).unwrap();
        items.insert(item_index, item_bounds);
    }

    for item_index in (0..160).step_by(3) {
        let x = item_index as f64 * 1.1 - 40.0;
        let item_bounds = bounds(x, -4.0, x + 2.25, 4.0);
        index.update(item_index, item_bounds).unwrap();
        items.insert(item_index, item_bounds);
    }

    for item_index in (0..160).step_by(5) {
        index.remove(item_index).unwrap();
        items.remove(&item_index);
    }

    for offset in 0..80 {
        let item_index = 1000 + offset;
        let x = offset as f64 * 2.0 - 25.0;
        let item_bounds = bounds(x, 10.0, x + 0.75, 11.0);
        index.insert(item_index, item_bounds).unwrap();
        items.insert(item_index, item_bounds);
    }

    let padding = 0.35;
    for step in 0..40 {
        let x = step as f64 * 11.0 - 75.0;
        let query = bounds(x, -5.0, x + 18.0, 12.0);
        assert_eq!(
            index.query_bounds(query, padding).unwrap(),
            brute_query(&items, query, padding),
            "query step {step}"
        );
    }

    index.rebuild();
    for step in 0..40 {
        let x = step as f64 * 11.0 - 75.0;
        let query = bounds(x, -5.0, x + 18.0, 12.0);
        assert_eq!(
            index.query_bounds(query, padding).unwrap(),
            brute_query(&items, query, padding),
            "rebuilt query step {step}"
        );
    }
}

#[test]
fn dynamic_candidate_pairs_match_brute_force_with_sparse_ids() {
    let mut left = DynamicSpatialIndex::new();
    let mut left_items = BTreeMap::new();
    for (item_index, item_bounds) in [
        (90, bounds(0.0, 0.0, 4.0, 4.0)),
        (5, bounds(3.5, 3.5, 7.0, 7.0)),
        (400, bounds(20.0, 0.0, 22.0, 2.0)),
        (17, bounds(21.5, 1.5, 24.0, 3.0)),
    ] {
        left.insert(item_index, item_bounds).unwrap();
        left_items.insert(item_index, item_bounds);
    }

    let mut right = DynamicSpatialIndex::new();
    let mut right_items = BTreeMap::new();
    for (item_index, item_bounds) in [
        (700, bounds(1.0, 1.0, 2.0, 2.0)),
        (3, bounds(23.5, 2.0, 25.0, 4.0)),
        (81, bounds(100.0, 100.0, 101.0, 101.0)),
    ] {
        right.insert(item_index, item_bounds).unwrap();
        right_items.insert(item_index, item_bounds);
    }

    let padding = 0.25;
    assert_eq!(
        left.self_candidate_pairs(padding).unwrap(),
        brute_self_pairs(&left_items, padding)
    );
    assert_eq!(
        left.cross_candidate_pairs(&right, padding).unwrap(),
        brute_cross_pairs(&left_items, &right_items, padding)
    );
}

#[test]
fn repeated_updates_of_one_item_do_not_duplicate_results() {
    let mut index = DynamicSpatialIndex::new();
    index.insert(11, bounds(0.0, 0.0, 1.0, 1.0)).unwrap();

    for step in 0..100 {
        let x = step as f64 * 0.1;
        index.update(11, bounds(x, 0.0, x + 1.0, 1.0)).unwrap();
        assert_eq!(
            index
                .query_bounds(bounds(x, -1.0, x + 1.0, 2.0), 0.0)
                .unwrap(),
            vec![11]
        );
    }
}

#[test]
fn dynamic_index_remains_stable_at_large_coordinates() {
    let offset = 1.0e12;
    let mut index = DynamicSpatialIndex::new();
    index
        .insert(8, bounds(offset, offset, offset + 10.0, offset + 10.0))
        .unwrap();
    index
        .insert(
            2,
            bounds(offset + 100.0, offset, offset + 110.0, offset + 10.0),
        )
        .unwrap();

    assert_eq!(
        index
            .query_bounds(
                bounds(offset + 5.0, offset + 5.0, offset + 6.0, offset + 6.0,),
                0.0,
            )
            .unwrap(),
        vec![8]
    );

    index
        .update(
            2,
            bounds(offset + 8.0, offset + 8.0, offset + 12.0, offset + 12.0),
        )
        .unwrap();
    assert_eq!(index.self_candidate_pairs(0.0).unwrap(), vec![(2, 8)]);
}

#[test]
fn invalid_dynamic_mutations_are_rejected() {
    let mut index = DynamicSpatialIndex::new();
    index.insert(1, bounds(0.0, 0.0, 1.0, 1.0)).unwrap();

    assert_eq!(
        index.insert(1, bounds(2.0, 2.0, 3.0, 3.0)),
        Err(CoreError::InvalidGeometry)
    );
    assert_eq!(
        index.insert(2, Bounds::Empty),
        Err(CoreError::InvalidGeometry)
    );
    assert_eq!(
        index.insert(
            2,
            Bounds::Finite {
                min: Point2::new(f64::NAN, 0.0),
                max: Point2::new(1.0, 1.0),
            },
        ),
        Err(CoreError::InvalidNumber)
    );
    assert_eq!(
        index.update(1, bounds(2.0, 0.0, 1.0, 1.0)),
        Err(CoreError::InvalidGeometry)
    );
    assert_eq!(
        index.update(99, bounds(0.0, 0.0, 1.0, 1.0)),
        Err(CoreError::InvalidGeometry)
    );
    assert_eq!(index.remove(99), Err(CoreError::InvalidGeometry));
    assert_eq!(
        index.query_bounds(bounds(0.0, 0.0, 1.0, 1.0), -1.0),
        Err(CoreError::InvalidGeometry)
    );
    assert_eq!(
        index.query_bounds(bounds(0.0, 0.0, 1.0, 1.0), f64::NAN),
        Err(CoreError::InvalidNumber)
    );
}

#[test]
fn clear_resets_dynamic_index_state() {
    let mut index = DynamicSpatialIndex::new();
    for item_index in 0..50 {
        let x = item_index as f64;
        index
            .insert(item_index, bounds(x, 0.0, x + 1.0, 1.0))
            .unwrap();
    }

    index.clear();

    assert!(index.is_empty());
    assert_eq!(index.len(), 0);
    assert!(
        index
            .query_bounds(bounds(-10.0, -10.0, 100.0, 100.0), 0.0)
            .unwrap()
            .is_empty()
    );
}
