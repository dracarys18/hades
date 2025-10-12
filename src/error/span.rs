use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn into_range(&self) -> Range<usize> {
        Range {
            start: self.start,
            end: self.end,
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::new(0, 0)
    }
}
