use crate::Subpath;

pub struct Path {
    subpaths: Vec<Subpath>,
}

impl Path {
    pub const fn new() -> Self {
        Self { subpaths: Vec::new() }
    }

    pub(crate) fn from_subpaths(subpaths: Vec<Subpath>) -> Self {
        Self { subpaths }
    }

    pub fn subpaths(&self) -> &[Subpath] {
        &self.subpaths
    }

    pub fn is_empty(&self) -> bool {
        self.subpaths.is_empty()
    }
}
