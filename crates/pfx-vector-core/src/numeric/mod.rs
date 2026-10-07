mod integration;
mod predicates;
mod roots;
mod robustness;
pub use integration::adaptive_simpson;
pub use predicates::{
    Orientation, orientation
};
pub use roots::{
    solve_cubic, solve_linear, solve_quadratic
};
pub use robustness::{
    clamp_unit, dedup_sorted
};
