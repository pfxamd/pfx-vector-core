use core::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreError {
    InvalidNumber,
    InvalidGeometry,
    SingularTransform,
    DegenerateOperation,
    NonConvergent,
    ToleranceNotMet,
    IterationLimit,
    UnsupportedCase,
}

pub type CoreResult<T> = Result<T, CoreError>;

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for CoreError {}
