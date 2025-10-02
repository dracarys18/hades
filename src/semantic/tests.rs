#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Program, Stmt, Types};
    use crate::error::Span;
    use crate::tokens::Ident;
    use indexmap::IndexMap;

    fn create_test_span() -> Span {
        Span::new(0, 1)
    }

    fn create_ident(name: &str) -> Ident {
        Ident::new(name.to_string())
    }

    #[test]
    fn test_basic_let_statement() {
        let program = Program::new(vec![
            Stmt::Let {
                name: create_ident("x"),
                declared_type: Some(Types::Int),
                value: Expr::Number(42),
                span: create_test_span(),
            }
        ]);

        let mut analyzer = Analyzer::new(&program);
        assert!(analyzer.analyze().is_ok());
    }

    #[test]
    fn test_type_mismatch_in_let() {
        let program = Program::new(vec![
            Stmt::Let {
                name: create_ident("x"),
                declared_type: Some(Types::Int),
                value: Expr::Boolean(true),
                span: create_test_span(),
            }
        ]);

        let mut analyzer = Analyzer::new(&program);
        assert!(analyzer.analyze().is_err());
    }

    #[test]
    fn test_undefined_variable() {
        let program = Program::new(vec![
            Stmt::Expr {
                expr: Expr::Ident(create_ident("undefined_var")),
                span: create_test_span(),
            }
        ]);

        let mut analyzer = Analyzer::new(&program);
        assert!(analyzer.analyze().is_err());
    }

    #[test]
    fn test_if_statement_with_boolean_condition() {
        let program = Program::new(vec![
            Stmt::If {
                cond: Expr::Boolean(true),
                then_branch: Program::new(vec![]),
                else_branch: None,
                span: create_test_span(),
            }
        ]);

        let mut analyzer = Analyzer::new(&program);
        assert!(analyzer.analyze().is_ok());
    }

    #[test]
    fn test_if_statement_with_non_boolean_condition() {
        let program = Program::new(vec![
            Stmt::If {
                cond: Expr::Number(42),
                then_branch: Program::new(vec![]),
                else_branch: None,
                span: create_test_span(),
            }
        ]);

        let mut analyzer = Analyzer::new(&program);
        let result = analyzer.analyze();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("If condition must be boolean"));
    }

    #[test]
    fn test_while_statement_with_boolean_condition() {
        let program = Program::new(vec![
            Stmt::While {
                cond: Expr::Boolean(false),
                body: Program::new(vec![]),
                span: create_test_span(),
            }
        ]);

        let mut analyzer = Analyzer::new(&program);
        assert!(analyzer.analyze().is_ok());
    }

    #[test]
    fn test_while_statement_with_non_boolean_condition() {
        let program = Program::new(vec![
            Stmt::While {
                cond: Expr::String("not boolean".to_string()),
                body: Program::new(vec![]),
                span: create_test_span(),
            }
        ]);

        let mut analyzer = Analyzer::new(&program);
        let result = analyzer.analyze();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("While loop condition must be boolean"));
    }

    #[test]
    fn test_function_definition_and_call() {
        let program = Program::new(vec![
            Stmt::FuncDef {
                name: create_ident("add"),
                params: vec![
                    (create_ident("a"), Types::Int),
                    (create_ident("b"), Types::Int),
                ],
                return_type: Types::Int,
                body: Program::new(vec![]),
                span: create_test_span(),
            },
            Stmt::Expr {
                expr: Expr::Call {
                    func: create_ident("add"),
                    args: vec![Expr::Number(1), Expr::Number(2)],
                },
                span: create_test_span(),
            }
        ]);

        let mut analyzer = Analyzer::new(&program);
        assert!(analyzer.analyze().is_ok());
    }

    #[test]
    fn test_function_call_wrong_argument_count() {
        let program = Program::new(vec![
            Stmt::FuncDef {
                name: create_ident("add"),
                params: vec![
                    (create_ident("a"), Types::Int),
                    (create_ident("b"), Types::Int),
                ],
                return_type: Types::Int,
                body: Program::new(vec![]),
                span: create_test_span(),
            },
            Stmt::Expr {
                expr: Expr::Call {
                    func: create_ident("add"),
                    args: vec![Expr::Number(1)], // Only one argument
                },
                span: create_test_span(),
            }
        ]);

        let mut analyzer = Analyzer::new(&program);
        let result = analyzer.analyze();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expects 2 arguments, got 1"));
    }

    #[test]
    fn test_function_call_wrong_argument_type() {
        let program = Program::new(vec![
            Stmt::FuncDef {
                name: create_ident("print_int"),
                params: vec![(create_ident("x"), Types::Int)],
                return_type: Types::Void,
                body: Program::new(vec![]),
                span: create_test_span(),
            },
            Stmt::Expr {
                expr: Expr::Call {
                    func: create_ident("print_int"),
                    args: vec![Expr::Boolean(true)], // Wrong type
                },
                span: create_test_span(),
            }
        ]);

        let mut analyzer = Analyzer::new(&program);
        let result = analyzer.analyze();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expects type"));
    }

    #[test]
    fn test_undefined_function_call() {
        let program = Program::new(vec![
            Stmt::Expr {
                expr: Expr::Call {
                    func: create_ident("undefined_func"),
                    args: vec![],
                },
                span: create_test_span(),
            }
        ]);

        let mut analyzer = Analyzer::new(&program);
        let result = analyzer.analyze();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined function"));
    }

    #[test]
    fn test_struct_definition() {
        let mut fields = IndexMap::new();
        fields.insert(create_ident("x"), Types::Int);
        fields.insert(create_ident("y"), Types::Float);

        let program = Program::new(vec![
            Stmt::StructDef {
                name: create_ident("Point"),
                fields,
                span: create_test_span(),
            }
        ]);

        let mut analyzer = Analyzer::new(&program);
        assert!(analyzer.analyze().is_ok());
    }

    #[test]
    fn test_variable_scope() {
        let program = Program::new(vec![
            Stmt::Let {
                name: create_ident("x"),
                declared_type: Some(Types::Int),
                value: Expr::Number(1),
                span: create_test_span(),
            },
            Stmt::If {
                cond: Expr::Boolean(true),
                then_branch: Program::new(vec![
                    Stmt::Let {
                        name: create_ident("y"),
                        declared_type: Some(Types::Int),
                        value: Expr::Number(2),
                        span: create_test_span(),
                    }
                ]),
                else_branch: None,
                span: create_test_span(),
            },
            // This should fail because 'y' is not in scope here
            Stmt::Expr {
                expr: Expr::Ident(create_ident("y")),
                span: create_test_span(),
            }
        ]);

        let mut analyzer = Analyzer::new(&program);
        let result = analyzer.analyze();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined variable"));
    }

    #[test]
    fn test_duplicate_function_definition() {
        let program = Program::new(vec![
            Stmt::FuncDef {
                name: create_ident("test"),
                params: vec![],
                return_type: Types::Void,
                body: Program::new(vec![]),
                span: create_test_span(),
            },
            Stmt::FuncDef {
                name: create_ident("test"), // Same name
                params: vec![],
                return_type: Types::Int,
                body: Program::new(vec![]),
                span: create_test_span(),
            }
        ]);

        let mut analyzer = Analyzer::new(&program);
        let result = analyzer.analyze();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already defined"));
    }
}