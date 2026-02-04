use indexmap::IndexMap;
use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use std::path::{Path, PathBuf};

use crate::ast::{Import, Program, Stmt};
use crate::module::error::ModuleError;
use crate::module::loader::{Loader, Module};
use crate::module::path::ModulePath;
use crate::module::resolver::Resolver;

pub struct Registry {
    modules: IndexMap<ModulePath, Module>,
    dag: Graph<ModulePath, ()>,
    node_map: IndexMap<ModulePath, NodeIndex>,
    loader: Loader,
    resolver: Resolver,
}

impl Registry {
    pub fn new(project_dir: impl AsRef<Path>, stdlib_dir: impl AsRef<Path>) -> Self {
        let resolver = Resolver::new(project_dir, stdlib_dir);
        let loader = Loader::new(resolver.clone());

        Self {
            modules: IndexMap::new(),
            dag: Graph::new(),
            node_map: IndexMap::new(),
            loader,
            resolver,
        }
    }

    pub fn load(&mut self, entry_path: impl AsRef<Path>) -> Result<Program, ModuleError> {
        let entry_path = entry_path.as_ref();

        let main_file = if entry_path.is_dir() {
            entry_path.join("main.hd")
        } else {
            let filename = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| ModuleError::NotFound("Invalid file name".to_string()))?;

            if filename != "main.hd" {
                return Err(ModuleError::NotFound(format!(
                    "Entry file must be named 'main.hd', found '{}'",
                    filename
                )));
            }
            entry_path.to_path_buf()
        };

        if !main_file.exists() {
            return Err(ModuleError::NotFound(format!(
                "main.hd not found in: {}",
                entry_path.display()
            )));
        }

        self.load_entry(&main_file)?;

        let modules = self.compile_order()?;

        let mut merged_ast = Vec::new();
        for module in modules {
            for stmt in &module.ast {
                merged_ast.push(stmt.clone());
            }
        }

        Ok(Program::new(merged_ast))
    }

    fn load_entry(&mut self, entry_file: &PathBuf) -> Result<ModulePath, ModuleError> {
        let module = self.loader.load_from_file(entry_file.clone())?;
        let path = module.path.clone();
        self.load_module_recursive(module)?;
        Ok(path)
    }

    fn load_module_recursive(&mut self, module: Module) -> Result<(), ModuleError> {
        let module_path = module.path.clone();

        if self.modules.contains_key(&module_path) {
            return Ok(());
        }

        let node_idx = self.dag.add_node(module_path.clone());
        self.node_map.insert(module_path.clone(), node_idx);

        let imports: Vec<Import> = module
            .ast
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::Import(imp) => Some(imp.clone()),
                _ => None,
            })
            .collect();

        self.modules.insert(module_path.clone(), module);

        for import in imports {
            let dep_path = self.resolver.resolve(&import)?;

            if !self.modules.contains_key(&dep_path) {
                let dep_module = self.loader.load(&dep_path)?;
                self.load_module_recursive(dep_module)?;
            }

            let dep_idx = self.node_map[&dep_path];
            self.dag.add_edge(node_idx, dep_idx, ());

            if is_cyclic_directed(&self.dag) {
                return Err(ModuleError::CircularDependency);
            }
        }

        Ok(())
    }

    pub fn compile_order(&self) -> Result<Vec<&Module>, ModuleError> {
        let sorted = toposort(&self.dag, None).map_err(|_| ModuleError::CircularDependency)?;

        Ok(sorted
            .into_iter()
            .rev()
            .map(|idx| {
                let path = &self.dag[idx];
                &self.modules[path]
            })
            .collect())
    }

    pub fn get(&self, path: &ModulePath) -> Option<&Module> {
        self.modules.get(path)
    }
}
