use std::{ops::Range, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    range: std::ops::Range<usize>,
    file: PathBuf,
}

impl Span {
    pub fn new(filename: PathBuf, start: usize, end: usize) -> Self {
        let path = PathBuf::from(filename);
        let range = start..end;
        Self { range, file: path }
    }

    pub fn start(&self) -> usize {
        self.range.start
    }

    pub fn end(&self) -> usize {
        self.range.end
    }

    pub fn into_range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub fn to(&self, other: Span) -> Span {
        Span::new(
            self.file.clone(),
            self.start().min(other.start()),
            self.end().max(other.end()),
        )
    }

    pub fn shrink_to_lo(&self) -> Span {
        Span::new(self.file.clone(), self.start(), self.start())
    }

    pub fn shrink_to_hi(&self) -> Span {
        Span::new(self.file.clone(), self.end(), self.end())
    }

    pub fn contains(&self, pos: usize) -> bool {
        self.start() <= pos && pos < self.end()
    }

    pub fn is_empty(&self) -> bool {
        self.start() == self.end()
    }

    pub fn len(&self) -> usize {
        self.end().saturating_sub(self.start())
    }

    pub fn file(&self) -> &PathBuf {
        &self.file
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::new(PathBuf::from("dummyfile"), 0, 0)
    }
}
