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

pub struct EntryPath {
    pub path: PathBuf,
    pub project_dir: PathBuf,
}

impl EntryPath {
    pub fn new_checked(entry_path: PathBuf) -> Result<Self, ModuleError> {
        let (main_file, project_dir) = if entry_path.is_dir() {
            let main_file = entry_path.join("main.hd");
            (main_file, entry_path.clone())
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
            let project_dir = entry_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            (entry_path.to_path_buf(), project_dir)
        };
        Ok(Self {
            path: main_file,
            project_dir,
        })
    }
}

impl Registry {
    pub fn new(project_dir: impl AsRef<Path>) -> Self {
        let resolver = Resolver::new(project_dir);
        let loader = Loader::new(resolver.clone());

        Self {
            modules: IndexMap::new(),
            dag: Graph::new(),
            node_map: IndexMap::new(),
            loader,
            resolver,
        }
    }

    pub fn load(entry_path: impl AsRef<Path>) -> Result<Program, ModuleError> {
        let entry = EntryPath::new_checked(entry_path.as_ref().to_path_buf())?;
        let mut registry = Self::new(&entry.project_dir);
        registry.load_program(&entry.path)
    }

    fn load_program(&mut self, entry_path: &Path) -> Result<Program, ModuleError> {
        self.load_entry(&entry_path.to_path_buf())?;

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
