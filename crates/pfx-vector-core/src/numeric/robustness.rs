use crate::{
    Scalar, Tolerance
};
#[must_use]
pub fn clamp_unit(t: Scalar, tol: Tolerance) -> Option<Scalar> {
    if t < -tol.absolute || t > 1.0+tol.absolute {
        None
    } else if t.abs() <= tol.absolute {
        Some(0.0)
    } else if (1.0-t).abs() <= tol.absolute {
        Some(1.0)
    } else {
        Some(t.clamp(0.0, 1.0))
    }
}
pub fn dedup_sorted(values: &mut Vec<Scalar>, tol: Tolerance) {
    values.sort_by(|a, b|a.total_cmp(b));
    values.dedup_by(|a, b|tol.almost_eq(*a, *b));
}
