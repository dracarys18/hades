pub trait Backend {
    type Error;
}

pub trait Visit<B: Backend> {
    type Output;
    fn visit(&self, backend: &mut B) -> Result<Self::Output, B::Error>;
}
