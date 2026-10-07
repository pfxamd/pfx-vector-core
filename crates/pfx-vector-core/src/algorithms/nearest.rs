use crate::{
    CoreError, CoreResult, Path, PathLocation, Point2, Scalar, Tolerance, flatten_path, path_length,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClosestPointResult {
    pub point: Point2,
    pub distance: Scalar,
    pub distance_squared: Scalar,
    pub location: PathLocation,
}

fn closest_line(a: Point2, b: Point2, point: Point2) -> (Point2, Scalar) {
    let direction = b - a;
    let dd = direction.dot(direction);

    if dd == 0.0 {
        return (a, 0.0);
    }

    let t = ((point - a).dot(direction) / dd).clamp(0.0, 1.0);
    (a + direction * t, t)
}

pub fn closest_point(
    path: &Path,
    point: Point2,
    tolerance: Tolerance,
) -> CoreResult<ClosestPointResult> {
    if path.is_empty() {
        return Err(CoreError::DegenerateOperation);
    }

    let flattened = flatten_path(path, tolerance)?;
    let total = path_length(path, tolerance)?;
    let mut best = None;
    let mut distance_accumulated = 0.0;

    for (subpath_index, subpath) in flattened.iter().enumerate() {
        for (segment_index, window) in subpath.points.windows(2).enumerate() {
            let a = window[0];
            let b = window[1];
            let length = a.distance_to(b);
            let (candidate, t) = closest_line(a, b, point);
            let distance_squared = candidate.distance_squared_to(point);

            if best.map_or(true, |(value, _, _, _, _, _)| distance_squared < value) {
                best = Some((
                    distance_squared,
                    candidate,
                    subpath_index,
                    segment_index,
                    t,
                    distance_accumulated + t * length,
                ));
            }

            distance_accumulated += length;
        }
    }

    let (distance_squared, candidate, subpath_index, segment_index, t, distance) =
        best.ok_or(CoreError::DegenerateOperation)?;

    Ok(ClosestPointResult {
        point: candidate,
        distance: distance_squared.sqrt(),
        distance_squared,
        location: PathLocation {
            subpath_index,
            segment_index,
            t,
            distance: distance.min(total),
        },
    })
}
