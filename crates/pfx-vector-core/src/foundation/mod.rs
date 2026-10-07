mod error;
mod interval;
mod scalar;
mod tolerance;
pub use error::{CoreError, CoreResult};
pub use interval::Interval;
pub use scalar::{Scalar, is_finite, is_nearly_zero};
pub use tolerance::Tolerance;
