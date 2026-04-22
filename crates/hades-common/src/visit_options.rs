#[derive(Debug, Clone, Copy)]
pub struct VisitOptions {
    pub ptr: bool,
}

impl VisitOptions {
    pub fn new() -> Self {
        Self { ptr: false }
    }

    pub fn with_ptr(mut self, ptr: bool) -> Self {
        self.ptr = ptr;
        self
    }
}

impl Default for VisitOptions {
    fn default() -> Self {
        Self::new()
    }
}
