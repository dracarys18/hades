#[derive(PartialEq, Clone, Debug, Eq, Hash)]
pub struct Ident(pub String);

impl Ident {
    pub fn new(name: String) -> Self {
        Self(name)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
