mod integration;
mod predicates;
mod robustness;
mod roots;
pub use integration::adaptive_simpson;
pub use predicates::{Orientation, orientation};
pub use robustness::{clamp_unit, dedup_sorted};
pub use roots::{solve_cubic, solve_linear, solve_quadratic};
