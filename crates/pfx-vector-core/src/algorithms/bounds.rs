use crate::{Bounds, CoreResult, Path, Tolerance, Transform2D, transform_path};
#[must_use]
pub fn path_bounds(path: &Path) -> Bounds {
    let mut bounds = Bounds::Empty;
    for subpath in path.subpaths() {
        for segment in subpath.segments() {
            bounds = bounds.union(segment.bounds());
        }
    }
    bounds
}
pub fn transformed_path_bounds(
    path: &Path,
    transform: Transform2D,
    tolerance: Tolerance,
) -> CoreResult<Bounds> {
    Ok(path_bounds(&transform_path(path, transform, tolerance)?))
}
