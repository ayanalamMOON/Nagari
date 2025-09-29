#[cfg(test)]
mod tests {
    use super::*;
    use crate::nagari_parser::ast::*;

    fn create_test_generator() -> CodeGenerator {
        CodeGenerator::new()
    }

    fn create_simple_program(statements: Vec<Statement>) -> Program {
        Program { statements }
    }

    #[test]
    fn test_import_simple() {
        let mut generator = create_test_generator();
        let import_stmt = Statement::Import {
            source: "math".to_string(),
            items: vec![],
        };

        assert!(generator.compile_statement(&import_stmt).is_ok());

        // Verify that import info was recorded
        assert_eq!(generator.imports.len(), 1);
        assert_eq!(generator.imports[0].module_name, "math");
        assert!(!generator.imports[0].is_star_import);

        // Verify bytecode was generated
        assert!(!generator.instructions.is_empty());
    }

    #[test]
    fn test_import_from() {
        let mut generator = create_test_generator();
        let import_stmt = Statement::Import {
            source: "math".to_string(),
            items: vec![
                ImportItem {
                    name: "sqrt".to_string(),
                    alias: None,
                },
                ImportItem {
                    name: "pi".to_string(),
                    alias: Some("PI".to_string()),
                },
            ],
        };

        assert!(generator.compile_statement(&import_stmt).is_ok());

        // Verify import info
        assert_eq!(generator.imports.len(), 1);
        assert_eq!(generator.imports[0].names, vec!["sqrt", "pi"]);
        assert_eq!(
            generator.imports[0].aliases,
            vec![None, Some("PI".to_string())]
        );
    }

    #[test]
    fn test_function_compilation() {
        let mut generator = create_test_generator();
        let func_stmt = Statement::Function {
            name: "add".to_string(),
            parameters: vec![
                FunctionParameter {
                    name: "a".to_string(),
                    type_annotation: None,
                    default_value: None,
                },
                FunctionParameter {
                    name: "b".to_string(),
                    type_annotation: None,
                    default_value: Some(Expression::Literal(Literal::Number(0.0))),
                },
            ],
            body: vec![Statement::Return(Some(Expression::Binary {
                left: Box::new(Expression::Identifier("a".to_string())),
                operator: BinaryOperator::Add,
                right: Box::new(Expression::Identifier("b".to_string())),
            }))],
            is_async: false,
            return_type: None,
        };

        assert!(generator.compile_statement(&func_stmt).is_ok());

        // Verify function was created
        assert!(generator.varnames.contains(&"add".to_string()));
        assert!(!generator.instructions.is_empty());
    }

    #[test]
    fn test_function_with_closure() {
        let mut generator = create_test_generator();
        let outer_var = "x".to_string();

        // Simulate having an outer variable
        generator.add_varname(outer_var.clone());

        let func_stmt = Statement::Function {
            name: "inner".to_string(),
            parameters: vec![],
            body: vec![Statement::Return(Some(Expression::Identifier(outer_var)))],
            is_async: false,
            return_type: None,
        };

        assert!(generator.compile_statement(&func_stmt).is_ok());
    }

    #[test]
    fn test_async_function() {
        let mut generator = create_test_generator();
        let func_stmt = Statement::Function {
            name: "async_func".to_string(),
            parameters: vec![],
            body: vec![Statement::Return(Some(Expression::Literal(
                Literal::Number(42.0),
            )))],
            is_async: true,
            return_type: None,
        };

        assert!(generator.compile_statement(&func_stmt).is_ok());

        // Verify async setup opcode was generated
        let has_async_setup = generator
            .instructions
            .iter()
            .any(|inst| matches!(inst.opcode, Opcode::SetupAsync));
        assert!(has_async_setup);
    }

    #[test]
    fn test_for_loop_compilation() {
        let mut generator = create_test_generator();
        let for_stmt = Statement::For {
            variable: "i".to_string(),
            iterable: Expression::List(vec![
                Expression::Literal(Literal::Number(1.0)),
                Expression::Literal(Literal::Number(2.0)),
                Expression::Literal(Literal::Number(3.0)),
            ]),
            body: vec![Statement::Expression(Expression::Call {
                function: Box::new(Expression::Identifier("print".to_string())),
                arguments: vec![Expression::Identifier("i".to_string())],
            })],
        };

        assert!(generator.compile_statement(&for_stmt).is_ok());

        // Verify loop opcodes were generated
        let has_get_iter = generator
            .instructions
            .iter()
            .any(|inst| matches!(inst.opcode, Opcode::GetIter));
        let has_for_iter = generator
            .instructions
            .iter()
            .any(|inst| matches!(inst.opcode, Opcode::ForIter));

        assert!(has_get_iter);
        assert!(has_for_iter);
    }

    #[test]
    fn test_for_loop_with_break() {
        let mut generator = create_test_generator();
        let for_stmt = Statement::For {
            variable: "i".to_string(),
            iterable: Expression::Identifier("items".to_string()),
            body: vec![Statement::If {
                condition: Expression::Binary {
                    left: Box::new(Expression::Identifier("i".to_string())),
                    operator: BinaryOperator::Greater,
                    right: Box::new(Expression::Literal(Literal::Number(5.0))),
                },
                then_body: vec![Statement::Break],
                else_body: None,
            }],
        };

        assert!(generator.compile_statement(&for_stmt).is_ok());

        // Verify that loop stack was properly managed
        assert!(generator.loop_stack.is_empty());
    }

    #[test]
    fn test_for_loop_with_continue() {
        let mut generator = create_test_generator();
        let for_stmt = Statement::For {
            variable: "i".to_string(),
            iterable: Expression::Identifier("items".to_string()),
            body: vec![
                Statement::If {
                    condition: Expression::Binary {
                        left: Box::new(Expression::Identifier("i".to_string())),
                        operator: BinaryOperator::Equal,
                        right: Box::new(Expression::Literal(Literal::Number(0.0))),
                    },
                    then_body: vec![Statement::Continue],
                    else_body: None,
                },
                Statement::Expression(Expression::Call {
                    function: Box::new(Expression::Identifier("print".to_string())),
                    arguments: vec![Expression::Identifier("i".to_string())],
                }),
            ],
        };

        assert!(generator.compile_statement(&for_stmt).is_ok());
    }

    #[test]
    fn test_break_outside_loop_error() {
        let mut generator = create_test_generator();
        let break_stmt = Statement::Break;

        assert!(generator.compile_statement(&break_stmt).is_err());
    }

    #[test]
    fn test_continue_outside_loop_error() {
        let mut generator = create_test_generator();
        let continue_stmt = Statement::Continue;

        assert!(generator.compile_statement(&continue_stmt).is_err());
    }

    #[test]
    fn test_pattern_matching_literal() {
        let mut generator = create_test_generator();
        let match_stmt = MatchStatement {
            expression: Expression::Literal(Literal::Number(42.0)),
            cases: vec![
                MatchCase {
                    pattern: Pattern::Literal(ConstantValue::Int(42)),
                    guard: None,
                    body: vec![Statement::Return(Some(Expression::Literal(
                        Literal::String("matched".to_string()),
                    )))],
                },
                MatchCase {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: vec![Statement::Return(Some(Expression::Literal(
                        Literal::String("default".to_string()),
                    )))],
                },
            ],
        };

        assert!(generator.compile_match(&match_stmt).is_ok());
    }

    #[test]
    fn test_pattern_matching_identifier() {
        let mut generator = create_test_generator();
        let match_stmt = MatchStatement {
            expression: Expression::Identifier("value".to_string()),
            cases: vec![MatchCase {
                pattern: Pattern::Identifier("x".to_string()),
                guard: None,
                body: vec![Statement::Return(Some(Expression::Identifier(
                    "x".to_string(),
                )))],
            }],
        };

        assert!(generator.compile_match(&match_stmt).is_ok());

        // Verify that the identifier was added to varnames
        assert!(generator.varnames.contains(&"x".to_string()));
    }

    #[test]
    fn test_pattern_matching_tuple() {
        let mut generator = create_test_generator();
        let match_stmt = MatchStatement {
            expression: Expression::Tuple(vec![
                Expression::Literal(Literal::Number(1.0)),
                Expression::Literal(Literal::Number(2.0)),
            ]),
            cases: vec![MatchCase {
                pattern: Pattern::Tuple(vec![
                    Pattern::Identifier("a".to_string()),
                    Pattern::Identifier("b".to_string()),
                ]),
                guard: None,
                body: vec![Statement::Return(Some(Expression::Binary {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    operator: BinaryOperator::Add,
                    right: Box::new(Expression::Identifier("b".to_string())),
                }))],
            }],
        };

        assert!(generator.compile_match(&match_stmt).is_ok());
    }

    #[test]
    fn test_pattern_matching_with_guard() {
        let mut generator = create_test_generator();
        let match_stmt = MatchStatement {
            expression: Expression::Identifier("value".to_string()),
            cases: vec![MatchCase {
                pattern: Pattern::Identifier("x".to_string()),
                guard: Some(Expression::Binary {
                    left: Box::new(Expression::Identifier("x".to_string())),
                    operator: BinaryOperator::Greater,
                    right: Box::new(Expression::Literal(Literal::Number(0.0))),
                }),
                body: vec![Statement::Return(Some(Expression::Literal(
                    Literal::String("positive".to_string()),
                )))],
            }],
        };

        assert!(generator.compile_match(&match_stmt).is_ok());
    }

    #[test]
    fn test_constant_serialization() {
        let mut generator = create_test_generator();

        // Add various constants
        let int_idx = generator.add_constant(ConstantValue::Int(42));
        let float_idx = generator.add_constant(ConstantValue::Float(3.14));
        let string_idx = generator.add_constant(ConstantValue::String("hello".to_string()));
        let bool_idx = generator.add_constant(ConstantValue::Bool(true));
        let none_idx = generator.add_constant(ConstantValue::None);

        // Verify constants were added
        assert_eq!(generator.constants.len(), 5);
        assert_ne!(int_idx, float_idx);
        assert_ne!(float_idx, string_idx);

        // Test serialization
        let mut bytecode = Vec::new();
        for constant in &generator.constants {
            assert!(generator
                .serialize_constant(constant, &mut bytecode)
                .is_ok());
        }

        // Verify bytecode was generated
        assert!(!bytecode.is_empty());
    }

    #[test]
    fn test_scope_management() {
        let mut generator = create_test_generator();

        // Test entering and exiting function scope
        let scope_info = ScopeInfo {
            scope_type: ScopeType::Function,
            locals: std::collections::HashSet::new(),
            parent_locals: std::collections::HashSet::new(),
        };

        generator.scope_stack.push(scope_info);
        assert_eq!(generator.scope_stack.len(), 1);

        generator.scope_stack.pop();
        assert_eq!(generator.scope_stack.len(), 0);
    }

    #[test]
    fn test_variable_name_management() {
        let mut generator = create_test_generator();

        let var1_idx = generator.add_varname("variable1".to_string());
        let var2_idx = generator.add_varname("variable2".to_string());
        let var1_idx_duplicate = generator.add_varname("variable1".to_string());

        // First two should be different
        assert_ne!(var1_idx, var2_idx);

        // Adding same variable again should return same index
        assert_eq!(var1_idx, var1_idx_duplicate);

        // Verify variables are stored
        assert!(generator.varnames.contains(&"variable1".to_string()));
        assert!(generator.varnames.contains(&"variable2".to_string()));
    }

    #[test]
    fn test_full_program_compilation() {
        let program = Program {
            statements: vec![
                Statement::Import {
                    source: "math".to_string(),
                    items: vec![ImportItem {
                        name: "sqrt".to_string(),
                        alias: None,
                    }],
                },
                Statement::Function {
                    name: "calculate".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "x".to_string(),
                        type_annotation: None,
                        default_value: None,
                    }],
                    body: vec![
                        Statement::Let {
                            name: "result".to_string(),
                            value: Expression::Call {
                                function: Box::new(Expression::Identifier("sqrt".to_string())),
                                arguments: vec![Expression::Identifier("x".to_string())],
                            },
                        },
                        Statement::Return(Some(Expression::Identifier("result".to_string()))),
                    ],
                    is_async: false,
                    return_type: None,
                },
                Statement::Expression(Expression::Call {
                    function: Box::new(Expression::Identifier("calculate".to_string())),
                    arguments: vec![Expression::Literal(Literal::Number(16.0))],
                }),
            ],
        };

        let mut generator = create_test_generator();
        assert!(generator.generate(&program).is_ok());
    }

    #[test]
    fn test_nested_loops() {
        let mut generator = create_test_generator();
        let nested_for = Statement::For {
            variable: "i".to_string(),
            iterable: Expression::Identifier("outer_range".to_string()),
            body: vec![Statement::For {
                variable: "j".to_string(),
                iterable: Expression::Identifier("inner_range".to_string()),
                body: vec![Statement::If {
                    condition: Expression::Binary {
                        left: Box::new(Expression::Identifier("j".to_string())),
                        operator: BinaryOperator::Equal,
                        right: Box::new(Expression::Literal(Literal::Number(5.0))),
                    },
                    then_body: vec![Statement::Break],
                    else_body: None,
                }],
            }],
        };

        assert!(generator.compile_statement(&nested_for).is_ok());

        // Verify loop stack is properly cleaned up
        assert!(generator.loop_stack.is_empty());
    }

    #[test]
    fn test_error_handling() {
        let mut generator = create_test_generator();

        // Test invalid break
        assert!(generator.compile_statement(&Statement::Break).is_err());

        // Test invalid continue
        assert!(generator.compile_statement(&Statement::Continue).is_err());
    }
}
