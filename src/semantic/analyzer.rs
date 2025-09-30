use crate::ast::{self, Types};

pub struct Analyzer<'a> {
    idents: ast::IdentMap,
    ast: &'a ast::Program,
}

impl<'a> Analyzer<'a> {
    pub fn new(ast: &'a ast::Program) -> Self {
        Self {
            ast,
            idents: ast::IdentMap::empty(),
        }
    }

    pub fn analyze(&mut self) -> Result<(), String> {
        let ast = self.ast;

        for stmt in ast {
            match stmt {
                ast::Stmt::Let { name, value, .. } => {
                    self.check_let(stmt)?;
                }
                ast::Stmt::Return(value) => {
                    println!("Return statement: {:?}", value);
                }
                ast::Stmt::Expr(expression) => {
                    println!("Expression statement: {:?}", expression);
                }
                _ => {
                    return Err(format!("Unknown statement: {:?}", stmt));
                }
            }
        }
        Ok(())
    }

    pub fn check_let(&mut self, stmt: &ast::Stmt) -> Result<(), String> {
        if let ast::Stmt::Let { name, value, .. } = stmt {
            let typ = self.check_expr(value)?;
            self.idents.insert(name.clone(), typ)?;

            Ok(())
        } else {
            panic!("Not a let statement")
        }
    }

    pub fn check_expr(&mut self, expr: &ast::Expr) -> Result<Types, String> {
        match expr {
            ast::Expr::String(_) => Ok(Types::String),
            ast::Expr::Number(_) => Ok(Types::Int),
            ast::Expr::Boolean(_) => Ok(Types::Bool),
            ast::Expr::Float(_) => Ok(Types::Float),
            ast::Expr::StructInit { name, fields } => {
                let mut field_types = indexmap::IndexMap::new();
                for (field_name, field_expr) in fields {
                    let field_type = self.check_expr(field_expr)?;
                    field_types.insert(field_name.clone(), field_type);
                }
                Ok(Types::Struct {
                    name: name.clone(),
                    fields: field_types,
                })
            }
            _ => Ok(Types::Void),
        }
    }
}
