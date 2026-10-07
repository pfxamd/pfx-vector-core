use crate::Subpath;
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Path {
    subpaths: Vec<Subpath>
}
impl Path {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            subpaths: Vec::new()
        }
    }
    pub(crate) fn from_subpaths(subpaths: Vec<Subpath>) -> Self {
        Self {
            subpaths
        }
    }
    #[must_use]
    pub fn subpaths(&self) -> &[Subpath] {
        &self.subpaths
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.subpaths.iter().all(Subpath::is_empty)
    }
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.subpaths.iter().map(|s|s.segments().len()).sum()
    }
    #[must_use]
    pub fn reversed(&self) -> Self {
        Self::from_subpaths(self.subpaths.iter().rev().map(Subpath::reversed).collect())
    }
}
