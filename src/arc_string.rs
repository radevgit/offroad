#![allow(dead_code)]


use crate::Arc;

// A series of contiguous arc segments.
pub struct ArcString(pub Vec<Arc>);

// https://github.com/georust/geo/blob/main/geo-types/src/geometry/line_string.rs
pub struct ArcsIter<'a>(::core::slice::Iter<'a, Arc>);

impl<'a> Iterator for ArcsIter<'a> {
    type Item = &'a Arc;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> ExactSizeIterator for ArcsIter<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> DoubleEndedIterator for ArcsIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl ArcString {
    /// Instantiate Self from the raw content value
    pub fn new(value: Vec<Arc>) -> Self {
        Self(value)
    }

    /// Return an iterator yielding the arcs
    pub fn arcs(&self) -> ArcsIter {
        ArcsIter(self.0.iter())
    }

    pub fn arcs_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Arc> {
        self.0.iter_mut()
    }

}

#[cfg(test)]
mod test_arc_string {

    #[test]
    fn test_new() {}
}
