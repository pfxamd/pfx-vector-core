pub type Scalar = f64;

#[inline]
#[must_use]
pub fn is_finite(value: Scalar) -> bool {
    value.is_finite()
}

#[inline]
#[must_use]
pub fn is_nearly_zero(value: Scalar, scale: Scalar, tolerance: crate::Tolerance) -> bool {
    value.abs() <= tolerance.absolute + tolerance.relative * scale.abs().max(1.0)
}
