use crate::Scalar;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathLocation {
    pub subpath_index: usize, pub segment_index: usize, pub t: Scalar, pub distance: Scalar
}
