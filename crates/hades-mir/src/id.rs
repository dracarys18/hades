pub(crate) struct Id {
    index: usize,
}

impl Id {
    const INITIAL_INDEX: usize = 0;

    pub(crate) fn new() -> Self {
        Self {
            index: Self::INITIAL_INDEX,
        }
    }

    pub(crate) fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn next(&mut self) -> Self {
        let id = Self { index: self.index };
        self.index += 1;
        id
    }
}
