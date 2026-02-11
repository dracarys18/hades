use std::collections::HashMap;

pub struct Library {
    modules: HashMap<String, &'static str>,
}

impl Library {
    pub fn new() -> Self {
        let mut modules = HashMap::new();
        modules.insert("math".to_string(), include_str!("../std/math.hd"));
        Self { modules }
    }

    pub fn get_module(&self, name: &str) -> Option<&'static str> {
        self.modules.get(name).copied()
    }

    pub fn has_module(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }
}

impl Default for Library {
    fn default() -> Self {
        Self::new()
    }
}
