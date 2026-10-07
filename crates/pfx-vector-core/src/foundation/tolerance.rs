use super::Scalar;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tolerance {
    pub absolute: Scalar,
    pub relative: Scalar,
    pub angular: Scalar,
    pub flatness: Scalar,
}
