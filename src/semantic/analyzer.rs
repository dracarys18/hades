use crate::ast::{self, Types, TypeTable, TypeId};
use crate::tokens::{Op, Ident};
use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: Ident,
    pub params: Vec<(Ident, Types)>,
    pub return_type: Types,
}

pub struct FunctionRegistry {
    functions: IndexMap<Ident, FunctionSignature>,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        Self {
            functions: IndexMap::new(),
        }
    }
    
    pub fn register_function(&mut self, signature: FunctionSignature) -> Result<(), String> {
        if self.functions.contains_key(&signature.name) {
            return Err(format!("Function '{}' is already defined", signature.name));
        }
        self.functions.insert(signature.name.clone(), signature);
        Ok(())
    }
    
    pub fn get_function(&self, name: &Ident) -> Option<&FunctionSignature> {
        self.functions.get(name)
    }
    
    pub fn function_exists(&self, name: &Ident) -> bool {
        self.functions.contains_key(name)
    }
}

pub struct Analyzer<'a> {
    idents: ast::IdentMap,
    type_table: TypeTable,
    function_registry: FunctionRegistry,
    ast: &'a ast::Program,
}

impl<'a> Analyzer<'a> {
    pub fn new(ast: &'a ast::Program) -> Self {
        Self {
            ast,
            idents: ast::IdentMap::empty(),
            type_table: TypeTable::new(),
            function_registry: FunctionRegistry::new(),
        }
    }

    pub fn analyze(&mut self) -> Result<(), String> {
        let ast = self.ast;

        for stmt in ast {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }
    
    pub fn check_stmt(&mut self, stmt: &ast::Stmt) -> Result<(), String> {
        match stmt {
            ast::Stmt::Let { .. } => {
                self.check_let(stmt)?;
            }
            ast::Stmt::If { cond, then_branch, else_branch, .. } => {
                // Check that condition is boolean
                let cond_type = self.check_expr(cond)?;
                if !self.is_boolean_compatible(&cond_type) {
                    return Err(format!(
                        "If condition must be boolean, got {:?}. Consider using comparison operators (==, !=, <, >, etc.) or boolean expressions.", 
                        cond_type
                    ));
                }
                
                // Enter new scope for branches
                self.idents.enter_scope();
                for stmt in then_branch {
                    self.check_stmt(stmt)?;
                }
                self.idents.exit_scope();
                
                if let Some(else_stmts) = else_branch {
                    self.idents.enter_scope();
                    for stmt in else_stmts {
                        self.check_stmt(stmt)?;
                    }
                    self.idents.exit_scope();
                }
            }
            ast::Stmt::While { cond, body, .. } => {
                // Check that condition is boolean
                let cond_type = self.check_expr(cond)?;
                if !self.is_boolean_compatible(&cond_type) {
                    return Err(format!(
                        "While loop condition must be boolean, got {:?}. Consider using comparison operators (==, !=, <, >, etc.) or boolean expressions.", 
                        cond_type
                    ));
                }
                
                // Enter new scope for loop body
                self.idents.enter_scope();
                for stmt in body {
                    self.check_stmt(stmt)?;
                }
                self.idents.exit_scope();
            }
            ast::Stmt::For { init, cond, update, body, .. } => {
                // Enter new scope for the entire for loop
                self.idents.enter_scope();
                
                // Check initializer
                self.check_stmt(init)?;
                
                // Check condition is boolean
                let cond_type = self.check_expr(cond)?;
                if !self.is_boolean_compatible(&cond_type) {
                    return Err(format!(
                        "For loop condition must be boolean, got {:?}. Consider using comparison operators (==, !=, <, >, etc.) or boolean expressions.", 
                        cond_type
                    ));
                }
                
                // Check update expression
                self.check_expr(update)?;
                
                // Check body
                for stmt in body {
                    self.check_stmt(stmt)?;
                }
                
                self.idents.exit_scope();
            }
            ast::Stmt::FuncDef { name, params, return_type, body, .. } => {
                // Register the function signature
                let signature = FunctionSignature {
                    name: name.clone(),
                    params: params.clone(),
                    return_type: return_type.clone(),
                };
                self.function_registry.register_function(signature)?;
                
                // Enter new scope for function parameters and body
                self.idents.enter_scope();
                
                // Add parameters to scope
                for (param_name, param_type) in params {
                    let type_id = self.type_table.register_type(param_type.clone());
                    self.idents.insert(param_name.clone(), type_id)?;
                }
                
                // Check function body
                for stmt in body {
                    self.check_stmt(stmt)?;
                }
                
                self.idents.exit_scope();
            }
            ast::Stmt::StructDef { name, fields, .. } => {
                // Register the struct type
                let struct_type = Types::Struct {
                    name: name.clone(),
                    fields: fields.clone(),
                };
                self.type_table.register_type(struct_type);
            }
            ast::Stmt::Return { expr, .. } => {
                if let Some(expr) = expr {
                    self.check_expr(expr)?;
                }
            }
            ast::Stmt::Expr { expr, .. } => {
                self.check_expr(expr)?;
            }
            ast::Stmt::Continue { .. } => {
                // Continue statements are valid
            }
            ast::Stmt::Block { stmts, .. } => {
                self.idents.enter_scope();
                for stmt in stmts {
                    self.check_stmt(stmt)?;
                }
                self.idents.exit_scope();
            }
        }
        Ok(())
    }

    pub fn check_let(&mut self, stmt: &ast::Stmt) -> Result<(), String> {
        if let ast::Stmt::Let { name, declared_type, value, .. } = stmt {
            // Check the expression type
            let inferred_type = self.check_expr(value)?;
            
            // If there's a declared type, validate it matches the inferred type
            let final_type = if let Some(declared) = declared_type {
                if !self.types_compatible(&inferred_type, declared) {
                    return Err(format!(
                        "Type mismatch: variable '{}' declared as {:?} but assigned {:?}",
                        name, declared, inferred_type
                    ));
                }
                declared.clone()
            } else {
                inferred_type
            };
            
            // Register type and insert into identifier map
            let type_id = self.type_table.register_type(final_type);
            self.idents.insert(name.clone(), type_id)?;

            Ok(())
        } else {
            panic!("Not a let statement")
        }
    }
    
    fn types_compatible(&self, inferred: &Types, declared: &Types) -> bool {
        // For now, we require exact type matches
        // This can be extended later for type coercion rules
        inferred == declared
    }
    
    fn is_boolean_compatible(&self, typ: &Types) -> bool {
        matches!(typ, Types::Bool)
    }
    
    fn check_binary_operation(&self, left_type: &Types, op: &Op, right_type: &Types) -> Result<Types, String> {
        use crate::tokens::Op::*;
        
        match op {
            // Arithmetic operations
            Plus | Minus | Multiply | Divide => {
                match (left_type, right_type) {
                    (Types::Int, Types::Int) => Ok(Types::Int),
                    (Types::Float, Types::Float) => Ok(Types::Float),
                    (Types::Int, Types::Float) | (Types::Float, Types::Int) => Ok(Types::Float),
                    _ => Err(format!("Invalid arithmetic operation: {:?} {:?} {:?}", left_type, op, right_type))
                }
            }
            // Comparison operations
            EqualEqual | BangEqual | Greater | GreaterEqual | Less | LessEqual => {
                if self.types_compatible(left_type, right_type) {
                    Ok(Types::Bool)
                } else {
                    Err(format!("Cannot compare {:?} with {:?}", left_type, right_type))
                }
            }
            // Logical operations
            BooleanAnd | BooleanOr => {
                match (left_type, right_type) {
                    (Types::Bool, Types::Bool) => Ok(Types::Bool),
                    _ => Err(format!("Logical operations require boolean operands, got {:?} {:?} {:?}", left_type, op, right_type))
                }
            }
            _ => Err(format!("Unsupported binary operation: {:?}", op))
        }
    }
    
    fn check_unary_operation(&self, op: &Op, expr_type: &Types) -> Result<Types, String> {
        use crate::tokens::Op::*;
        
        match op {
            Minus => {
                match expr_type {
                    Types::Int => Ok(Types::Int),
                    Types::Float => Ok(Types::Float),
                    _ => Err(format!("Cannot apply unary minus to {:?}", expr_type))
                }
            }
            Bang => {
                match expr_type {
                    Types::Bool => Ok(Types::Bool),
                    _ => Err(format!("Cannot apply logical not to {:?}", expr_type))
                }
            }
            _ => Err(format!("Unsupported unary operation: {:?}", op))
        }
    }
    
    fn check_function_call(&mut self, func_name: &crate::tokens::Ident, args: &[ast::Expr]) -> Result<Types, String> {
        // Check if function exists
        let signature = self.function_registry.get_function(func_name)
            .ok_or_else(|| format!("Undefined function: {}", func_name))?;
        
        // Check argument count
        if args.len() != signature.params.len() {
            return Err(format!(
                "Function '{}' expects {} arguments, got {}",
                func_name, signature.params.len(), args.len()
            ));
        }
        
        // Check argument types
        for (i, arg) in args.iter().enumerate() {
            let arg_type = self.check_expr(arg)?;
            let expected_type = &signature.params[i].1;
            
            if !self.types_compatible(&arg_type, expected_type) {
                return Err(format!(
                    "Function '{}' argument {} expects type {:?}, got {:?}",
                    func_name, i + 1, expected_type, arg_type
                ));
            }
        }
        
        Ok(signature.return_type.clone())
    }
    
    fn check_compound_assignment(&self, var_type: &Types, op: &Op, value_type: &Types) -> Result<(), String> {
        use crate::tokens::Op::*;
        
        match op {
            PlusEqual | MinusEqual => {
                match (var_type, value_type) {
                    (Types::Int, Types::Int) | (Types::Float, Types::Float) => Ok(()),
                    (Types::Float, Types::Int) | (Types::Int, Types::Float) => Ok(()),
                    _ => Err(format!("Invalid compound assignment: {:?} {:?}= {:?}", var_type, op, value_type))
                }
            }
            _ => Err(format!("Unsupported compound assignment operator: {:?}", op))
        }
    }

    pub fn check_expr(&mut self, expr: &ast::Expr) -> Result<Types, String> {
        match expr {
            ast::Expr::String(_) => Ok(Types::String),
            ast::Expr::Number(_) => Ok(Types::Int),
            ast::Expr::Boolean(_) => Ok(Types::Bool),
            ast::Expr::Float(_) => Ok(Types::Float),
            ast::Expr::Ident(name) => {
                // Look up the identifier in the symbol table
                let type_id = self.idents.lookup(name)
                    .ok_or_else(|| format!("Undefined variable: {}", name))?;
                let typ = self.type_table.get(type_id)
                    .ok_or_else(|| format!("Type not found for variable: {}", name))?;
                Ok(typ.clone())
            }
            ast::Expr::Binary { left, op, right } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;
                self.check_binary_operation(&left_type, op, &right_type)
            }
            ast::Expr::Unary { op, expr } => {
                let expr_type = self.check_expr(expr)?;
                self.check_unary_operation(op, &expr_type)
            }
            ast::Expr::Call { func, args } => {
                self.check_function_call(func, args)
            }
            ast::Expr::Assign { name, op, value } => {
                // Check if the variable exists
                let var_type_id = self.idents.lookup(name)
                    .ok_or_else(|| format!("Undefined variable: {}", name))?;
                let var_type = self.type_table.get(var_type_id)
                    .ok_or_else(|| format!("Type not found for variable: {}", name))?;
                
                // Check the value type
                let value_type = self.check_expr(value)?;
                
                // Validate assignment compatibility
                if op.is_some() {
                    // For compound assignments like +=, -=
                    self.check_compound_assignment(var_type, op.as_ref().unwrap(), &value_type)?;
                } else {
                    // Simple assignment
                    if !self.types_compatible(&value_type, var_type) {
                        return Err(format!(
                            "Cannot assign {:?} to variable '{}' of type {:?}",
                            value_type, name, var_type
                        ));
                    }
                }
                
                Ok(var_type.clone())
            }
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
        }
    }
}
