use super::*;

mod map {
    use super::*;

    // MAP([1], 1)
    test_algebrize!(
        no_variables,
        method = algebrize_expression,
        in_implicit_type_conversion_context = false,
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Map(mir::MapExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Integer(1))]
                })),
                f: Box::new(mir::Expression::Literal(mir::LiteralValue::Integer(1))),
                is_nullable: false,
            })
        )),
        input =
            ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Map(ast::MapExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Literal(
                    ast::Literal::Integer(1)
                ))),
            })),
    );

    // MAP(foo.a, 1)
    test_algebrize!(
        nullable,
        method = algebrize_expression,
        in_implicit_type_conversion_context = false,
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Map(mir::MapExpr {
                array: Box::new(mir::Expression::FieldAccess(mir::FieldAccess {
                    expr: Box::new(mir::Expression::Reference(mir::ReferenceExpr {
                        key: ("foo", 0u16).into(),
                    })),
                    field: "a".to_string(),
                    is_nullable: true,
                })),
                f: Box::new(mir::Expression::Literal(mir::LiteralValue::Integer(1))),
                is_nullable: true,
            })
        )),
        input =
            ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Map(ast::MapExpr {
                array: Box::new(ast::Expression::Subpath(ast::SubpathExpr {
                    expr: Box::new(ast::Expression::Identifier("foo".to_string())),
                    subpath: "a".to_string(),
                })),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Literal(
                    ast::Literal::Integer(1)
                ))),
            })),
        env = map! {
            ("foo", 0u16).into() => Schema::Document( Document {
                keys: map! {
                    "a".into() => Schema::Array(Box::new(Schema::Atomic(Atomic::Integer))),
                },
                required: set!{},
                additional_properties: false,
                ..Default::default()
            }),
        },
    );

    // MAP([1], this)
    test_algebrize!(
        with_this_variable,
        method = algebrize_expression,
        in_implicit_type_conversion_context = false,
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Map(mir::MapExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Integer(1))]
                })),
                f: Box::new(mir::Expression::Variable(mir::Variable {
                    name: "this".to_string(),
                    is_nullable: false,
                })),
                is_nullable: false,
            })
        )),
        input =
            ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Map(ast::MapExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Identifier(
                    "this".to_string()
                ))),
            })),
    );

    // MAP([1, NULL], this)
    test_algebrize!(
        with_nullable_this_variable,
        method = algebrize_expression,
        in_implicit_type_conversion_context = false,
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Map(mir::MapExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![
                        mir::Expression::Literal(mir::LiteralValue::Integer(1)),
                        mir::Expression::Literal(mir::LiteralValue::Null),
                    ]
                })),
                f: Box::new(mir::Expression::Variable(mir::Variable {
                    name: "this".to_string(),
                    is_nullable: true,
                })),
                is_nullable: false,
            })
        )),
        input =
            ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Map(ast::MapExpr {
                array: Box::new(ast::Expression::Array(vec![
                    ast::Expression::Literal(ast::Literal::Integer(1)),
                    ast::Expression::Literal(ast::Literal::Null),
                ])),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Identifier(
                    "this".to_string()
                ))),
            })),
    );

    // MAP([1], this + this)
    test_algebrize!(
        with_this_variable_multiple_times,
        method = algebrize_expression,
        in_implicit_type_conversion_context = false,
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Map(mir::MapExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Integer(1))]
                })),
                f: Box::new(mir::Expression::ScalarFunction(
                    mir::ScalarFunctionApplication {
                        function: mir::ScalarFunction::Add,
                        args: vec![
                            mir::Expression::Variable(mir::Variable {
                                name: "this".to_string(),
                                is_nullable: false,
                            }),
                            mir::Expression::Variable(mir::Variable {
                                name: "this".to_string(),
                                is_nullable: false,
                            }),
                        ],
                        is_nullable: false,
                    }
                )),
                is_nullable: false,
            })
        )),
        input =
            ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Map(ast::MapExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Binary(
                    ast::BinaryExpr {
                        left: Box::new(ast::Expression::Identifier("this".to_string())),
                        op: ast::BinaryOp::Add,
                        right: Box::new(ast::Expression::Identifier("this".to_string())),
                    }
                ))),
            })),
    );

    // MAP([1], this + foo.this)
    test_algebrize!(
        with_this_variable_and_field_access,
        method = algebrize_expression,
        in_implicit_type_conversion_context = false,
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Map(mir::MapExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Integer(1))]
                })),
                f: Box::new(mir::Expression::ScalarFunction(
                    mir::ScalarFunctionApplication {
                        function: mir::ScalarFunction::Add,
                        args: vec![
                            mir::Expression::Variable(mir::Variable {
                                name: "this".to_string(),
                                is_nullable: false,
                            }),
                            mir::Expression::FieldAccess(mir::FieldAccess {
                                expr: Box::new(mir::Expression::Reference(mir::ReferenceExpr {
                                    key: ("foo", 0u16).into(),
                                })),
                                field: "this".to_string(),
                                is_nullable: false,
                            }),
                        ],
                        is_nullable: false,
                    }
                )),
                is_nullable: false,
            })
        )),
        input =
            ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Map(ast::MapExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Binary(
                    ast::BinaryExpr {
                        left: Box::new(ast::Expression::Identifier("this".to_string())),
                        op: ast::BinaryOp::Add,
                        right: Box::new(ast::Expression::Subpath(ast::SubpathExpr {
                            expr: Box::new(ast::Expression::Identifier("foo".to_string())),
                            subpath: "this".to_string(),
                        })),
                    }
                ))),
            })),
        env = map! {
            ("foo", 0u16).into() => Schema::Document( Document {
                keys: map! {
                    "this".into() => Schema::Atomic(Atomic::Integer),
                },
                required: set! { "this".to_string() },
                additional_properties: false,
                ..Default::default()
            }),
        },
    );

    test_algebrize!(
        invalid_array_arg,
        method = algebrize_expression,
        expected = Err(Error::HigherOrderFunctionWrapper {
            name: "Map",
            cause: HigherOrderFunctionErrorCause::ArrayArg,
            error: Box::new(Error::FieldNotFound(
                "foo".to_string(),
                None,
                ClauseType::Unintialized,
                0u16
            )),
        }),
        expected_error_code = 3035,
        input =
            ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Map(ast::MapExpr {
                array: Box::new(ast::Expression::Subpath(ast::SubpathExpr {
                    expr: Box::new(ast::Expression::Identifier("foo".to_string())),
                    subpath: "a".to_string(),
                })),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Literal(
                    ast::Literal::Integer(1)
                ))),
            })),
    );

    test_algebrize!(
        invalid_function_arg,
        method = algebrize_expression,
        expected = Err(Error::HigherOrderFunctionWrapper {
            name: "Map",
            cause: HigherOrderFunctionErrorCause::FunctionArg,
            error: Box::new(Error::FieldNotFound(
                "foo".to_string(),
                None,
                ClauseType::Unintialized,
                0u16
            )),
        }),
        expected_error_code = 3035,
        input =
            ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Map(ast::MapExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Binary(
                    ast::BinaryExpr {
                        left: Box::new(ast::Expression::Identifier("this".to_string())),
                        op: ast::BinaryOp::Add,
                        right: Box::new(ast::Expression::Subpath(ast::SubpathExpr {
                            expr: Box::new(ast::Expression::Identifier("foo".to_string())),
                            subpath: "this".to_string(),
                        })),
                    }
                ))),
            })),
    );
}
