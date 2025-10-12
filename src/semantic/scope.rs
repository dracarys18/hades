#[derive(Debug, Clone, PartialEq)]
pub struct Scope<T: Clone> {
    ele: Vec<T>,
}

impl<T: Clone> Scope<T> {
    pub fn empty() -> Self {
        Self { ele: vec![] }
    }

    pub fn enter_scope(&mut self, item: T) {
        self.ele.push(item);
    }

    pub fn exit_scope(&mut self) -> Option<T> {
        self.ele.pop()
    }

    pub fn current_scope(&self) -> Option<&T> {
        self.ele.last()
    }

    pub fn current_scope_mut(&mut self) -> Option<&mut T> {
        self.ele.last_mut()
    }

    pub fn on_scope<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&T) -> R,
    {
        self.current_scope().map(f)
    }

    pub fn lookup_scope<'a, F, G, R>(&'a self, cond: F, get: G) -> Option<&'a R>
    where
        F: FnMut(&&'a T) -> bool,
        G: FnOnce(&'a T) -> Option<&'a R>,
    {
        self.ele.iter().find(cond).and_then(get)
    }
}

impl<'a, T: Clone> IntoIterator for &'a Scope<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.ele.iter()
    }
}
