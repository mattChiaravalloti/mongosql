use super::*;

mod map {
    use super::*;

    // MAP([1], 1)
    test_algebrize!(
        no_variables,
        method = algebrize_expression,
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

mod filter {
    use super::*;

    // FILTER([1], true)
    test_algebrize!(
        no_variables,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Filter(mir::FilterExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Integer(1))]
                })),
                f: Box::new(mir::Expression::Literal(mir::LiteralValue::Boolean(true))),
                is_nullable: false,
            })
        )),
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Filter(
            ast::FilterExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Literal(
                    ast::Literal::Boolean(true)
                ))),
            }
        )),
    );

    // FILTER(foo.a, 1)
    test_algebrize!(
        nullable,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Filter(mir::FilterExpr {
                array: Box::new(mir::Expression::FieldAccess(mir::FieldAccess {
                    expr: Box::new(mir::Expression::Reference(mir::ReferenceExpr {
                        key: ("foo", 0u16).into(),
                    })),
                    field: "a".to_string(),
                    is_nullable: true,
                })),
                f: Box::new(mir::Expression::Literal(mir::LiteralValue::Boolean(true))),
                is_nullable: true,
            })
        )),
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Filter(
            ast::FilterExpr {
                array: Box::new(ast::Expression::Subpath(ast::SubpathExpr {
                    expr: Box::new(ast::Expression::Identifier("foo".to_string())),
                    subpath: "a".to_string(),
                })),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Literal(
                    ast::Literal::Boolean(true)
                ))),
            }
        )),
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

    // FILTER([true], this)
    test_algebrize!(
        with_this_variable,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Filter(mir::FilterExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Boolean(true))]
                })),
                f: Box::new(mir::Expression::Variable(mir::Variable {
                    name: "this".to_string(),
                    is_nullable: false,
                })),
                is_nullable: false,
            })
        )),
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Filter(
            ast::FilterExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Boolean(true)
                )])),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Identifier(
                    "this".to_string()
                ))),
            }
        )),
    );

    // FILTER([true, NULL], this)
    test_algebrize!(
        with_nullable_this_variable,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Filter(mir::FilterExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![
                        mir::Expression::Literal(mir::LiteralValue::Boolean(true)),
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
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Filter(
            ast::FilterExpr {
                array: Box::new(ast::Expression::Array(vec![
                    ast::Expression::Literal(ast::Literal::Boolean(true)),
                    ast::Expression::Literal(ast::Literal::Null),
                ])),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Identifier(
                    "this".to_string()
                ))),
            }
        )),
    );

    // FILTER([1], this = this)
    test_algebrize!(
        with_this_variable_multiple_times,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Filter(mir::FilterExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Integer(1))]
                })),
                f: Box::new(mir::Expression::ScalarFunction(
                    mir::ScalarFunctionApplication {
                        function: mir::ScalarFunction::Eq,
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
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Filter(
            ast::FilterExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Binary(
                    ast::BinaryExpr {
                        left: Box::new(ast::Expression::Identifier("this".to_string())),
                        op: ast::BinaryOp::Comparison(ast::ComparisonOp::Eq),
                        right: Box::new(ast::Expression::Identifier("this".to_string())),
                    }
                ))),
            }
        )),
    );

    // FILTER([1], this = foo.this)
    test_algebrize!(
        with_this_variable_and_field_access,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Filter(mir::FilterExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Integer(1))]
                })),
                f: Box::new(mir::Expression::ScalarFunction(
                    mir::ScalarFunctionApplication {
                        function: mir::ScalarFunction::Eq,
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
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Filter(
            ast::FilterExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Binary(
                    ast::BinaryExpr {
                        left: Box::new(ast::Expression::Identifier("this".to_string())),
                        op: ast::BinaryOp::Comparison(ast::ComparisonOp::Eq),
                        right: Box::new(ast::Expression::Subpath(ast::SubpathExpr {
                            expr: Box::new(ast::Expression::Identifier("foo".to_string())),
                            subpath: "this".to_string(),
                        })),
                    }
                ))),
            }
        )),
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
            name: "Filter",
            cause: HigherOrderFunctionErrorCause::ArrayArg,
            error: Box::new(Error::FieldNotFound(
                "foo".to_string(),
                None,
                ClauseType::Unintialized,
                0u16
            )),
        }),
        expected_error_code = 3035,
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Filter(
            ast::FilterExpr {
                array: Box::new(ast::Expression::Subpath(ast::SubpathExpr {
                    expr: Box::new(ast::Expression::Identifier("foo".to_string())),
                    subpath: "a".to_string(),
                })),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Literal(
                    ast::Literal::Boolean(true)
                ))),
            }
        )),
    );

    test_algebrize!(
        invalid_function_arg,
        method = algebrize_expression,
        expected = Err(Error::HigherOrderFunctionWrapper {
            name: "Filter",
            cause: HigherOrderFunctionErrorCause::FunctionArg,
            error: Box::new(Error::FieldNotFound(
                "foo".to_string(),
                None,
                ClauseType::Unintialized,
                0u16
            )),
        }),
        expected_error_code = 3035,
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Filter(
            ast::FilterExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Binary(
                    ast::BinaryExpr {
                        left: Box::new(ast::Expression::Identifier("this".to_string())),
                        op: ast::BinaryOp::Comparison(ast::ComparisonOp::Eq),
                        right: Box::new(ast::Expression::Subpath(ast::SubpathExpr {
                            expr: Box::new(ast::Expression::Identifier("foo".to_string())),
                            subpath: "this".to_string(),
                        })),
                    }
                ))),
            }
        )),
    );
}

mod reduce {
    use super::*;

    // REDUCE([1], 1, 1)
    test_algebrize!(
        no_variables,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Reduce(mir::ReduceExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Integer(1))]
                })),
                init_value: Box::new(mir::Expression::Literal(mir::LiteralValue::Integer(1))),
                f: Box::new(mir::Expression::Literal(mir::LiteralValue::Integer(1))),
                is_nullable: false,
            })
        )),
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Reduce(
            ast::ReduceExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                init_value: Box::new(ast::Expression::Literal(ast::Literal::Integer(1))),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Literal(
                    ast::Literal::Integer(1)
                ))),
            }
        )),
    );

    // REDUCE(foo.a, 1, 1)
    test_algebrize!(
        nullable_because_of_array,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Reduce(mir::ReduceExpr {
                array: Box::new(mir::Expression::FieldAccess(mir::FieldAccess {
                    expr: Box::new(mir::Expression::Reference(mir::ReferenceExpr {
                        key: ("foo", 0u16).into(),
                    })),
                    field: "a".to_string(),
                    is_nullable: true,
                })),
                init_value: Box::new(mir::Expression::Literal(mir::LiteralValue::Integer(1))),
                f: Box::new(mir::Expression::Literal(mir::LiteralValue::Integer(1))),
                is_nullable: true,
            })
        )),
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Reduce(
            ast::ReduceExpr {
                array: Box::new(ast::Expression::Subpath(ast::SubpathExpr {
                    expr: Box::new(ast::Expression::Identifier("foo".to_string())),
                    subpath: "a".to_string(),
                })),
                init_value: Box::new(ast::Expression::Literal(ast::Literal::Integer(1))),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Literal(
                    ast::Literal::Integer(1)
                ))),
            }
        )),
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

    // REDUCE([1], 1, this)
    test_algebrize!(
        with_this_variable,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Reduce(mir::ReduceExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Integer(1))]
                })),
                init_value: Box::new(mir::Expression::Literal(mir::LiteralValue::Integer(1))),
                f: Box::new(mir::Expression::Variable(mir::Variable {
                    name: "this".to_string(),
                    is_nullable: false,
                })),
                is_nullable: false,
            })
        )),
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Reduce(
            ast::ReduceExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                init_value: Box::new(ast::Expression::Literal(ast::Literal::Integer(1))),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Identifier(
                    "this".to_string()
                ))),
            }
        )),
    );

    // REDUCE([1, NULL], 1, this)
    test_algebrize!(
        with_nullable_this_variable,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Reduce(mir::ReduceExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![
                        mir::Expression::Literal(mir::LiteralValue::Integer(1)),
                        mir::Expression::Literal(mir::LiteralValue::Null),
                    ]
                })),
                init_value: Box::new(mir::Expression::Literal(mir::LiteralValue::Integer(1))),
                f: Box::new(mir::Expression::Variable(mir::Variable {
                    name: "this".to_string(),
                    is_nullable: true,
                })),
                is_nullable: true,
            })
        )),
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Reduce(
            ast::ReduceExpr {
                array: Box::new(ast::Expression::Array(vec![
                    ast::Expression::Literal(ast::Literal::Integer(1)),
                    ast::Expression::Literal(ast::Literal::Null),
                ])),
                init_value: Box::new(ast::Expression::Literal(ast::Literal::Integer(1))),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Identifier(
                    "this".to_string()
                ))),
            }
        )),
    );

    // REDUCE([NULL], 1, value)
    test_algebrize!(
        with_value_variable,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Reduce(mir::ReduceExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Null)]
                })),
                init_value: Box::new(mir::Expression::Literal(mir::LiteralValue::Integer(1))),
                f: Box::new(mir::Expression::Variable(mir::Variable {
                    name: "value".to_string(),
                    is_nullable: false,
                })),
                is_nullable: false,
            })
        )),
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Reduce(
            ast::ReduceExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Null
                )])),
                init_value: Box::new(ast::Expression::Literal(ast::Literal::Integer(1))),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Identifier(
                    "value".to_string()
                ))),
            }
        )),
    );

    // REDUCE([1], null, value)
    test_algebrize!(
        with_nullable_value_variable_because_of_init_value,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Reduce(mir::ReduceExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Integer(1)),]
                })),
                init_value: Box::new(mir::Expression::Literal(mir::LiteralValue::Null)),
                f: Box::new(mir::Expression::Variable(mir::Variable {
                    name: "value".to_string(),
                    is_nullable: true,
                })),
                is_nullable: true,
            })
        )),
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Reduce(
            ast::ReduceExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                ),])),
                init_value: Box::new(ast::Expression::Literal(ast::Literal::Null)),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Identifier(
                    "value".to_string()
                ))),
            }
        )),
    );

    // REDUCE([1], 1, value + null)
    test_algebrize!(
        with_nullable_value_variable_because_of_function_argument,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Reduce(mir::ReduceExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Integer(1)),]
                })),
                init_value: Box::new(mir::Expression::Literal(mir::LiteralValue::Integer(1))),
                f: Box::new(mir::Expression::ScalarFunction(
                    mir::ScalarFunctionApplication {
                        function: mir::ScalarFunction::Add,
                        args: vec![
                            mir::Expression::Variable(mir::Variable {
                                name: "value".to_string(),
                                is_nullable: true,
                            }),
                            mir::Expression::Literal(mir::LiteralValue::Null),
                        ],
                        is_nullable: true,
                    }
                )),
                is_nullable: true,
            })
        )),
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Reduce(
            ast::ReduceExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                ),])),
                init_value: Box::new(ast::Expression::Literal(ast::Literal::Integer(1))),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Binary(
                    ast::BinaryExpr {
                        left: Box::new(ast::Expression::Identifier("value".to_string())),
                        op: ast::BinaryOp::Add,
                        right: Box::new(ast::Expression::Literal(ast::Literal::Null)),
                    }
                ))),
            }
        )),
    );

    // REDUCE([1], field, this + value + this + value)
    test_algebrize!(
        with_variables_multiple_times,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Reduce(mir::ReduceExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Integer(1))]
                })),
                init_value: Box::new(mir::Expression::FieldAccess(mir::FieldAccess {
                    expr: Box::new(mir::Expression::Reference(mir::ReferenceExpr {
                        key: ("foo", 0u16).into(),
                    })),
                    field: "field".to_string(),
                    is_nullable: true,
                })),
                f: Box::new(mir::Expression::ScalarFunction(
                    mir::ScalarFunctionApplication {
                        function: mir::ScalarFunction::Add,
                        args: vec![
                            mir::Expression::Variable(mir::Variable {
                                name: "this".to_string(),
                                is_nullable: false,
                            }),
                            mir::Expression::ScalarFunction(mir::ScalarFunctionApplication {
                                function: mir::ScalarFunction::Add,
                                args: vec![
                                    mir::Expression::Variable(mir::Variable {
                                        name: "value".to_string(),
                                        is_nullable: true,
                                    }),
                                    mir::Expression::ScalarFunction(
                                        mir::ScalarFunctionApplication {
                                            function: mir::ScalarFunction::Add,
                                            args: vec![
                                                mir::Expression::Variable(mir::Variable {
                                                    name: "this".to_string(),
                                                    is_nullable: false,
                                                }),
                                                mir::Expression::Variable(mir::Variable {
                                                    name: "value".to_string(),
                                                    is_nullable: true,
                                                }),
                                            ],
                                            is_nullable: true,
                                        }
                                    )
                                ],
                                is_nullable: true,
                            })
                        ],
                        is_nullable: true,
                    }
                )),
                is_nullable: true,
            })
        )),
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Reduce(
            ast::ReduceExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                init_value: Box::new(ast::Expression::Subpath(ast::SubpathExpr {
                    expr: Box::new(ast::Expression::Identifier("foo".to_string())),
                    subpath: "field".to_string(),
                })),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Binary(
                    ast::BinaryExpr {
                        left: Box::new(ast::Expression::Identifier("this".to_string())),
                        op: ast::BinaryOp::Add,
                        right: Box::new(ast::Expression::Binary(ast::BinaryExpr {
                            left: Box::new(ast::Expression::Identifier("value".to_string())),
                            op: ast::BinaryOp::Add,
                            right: Box::new(ast::Expression::Binary(ast::BinaryExpr {
                                left: Box::new(ast::Expression::Identifier("this".to_string())),
                                op: ast::BinaryOp::Add,
                                right: Box::new(ast::Expression::Identifier("value".to_string())),
                            })),
                        })),
                    }
                ))),
            }
        )),
        env = map! {
            ("foo", 0u16).into() => Schema::Document( Document {
                keys: map! {
                    "field".into() => Schema::Atomic(Atomic::Integer),
                },
                required: set! { },
                additional_properties: false,
                ..Default::default()
            }),
        },
    );

    // REDUCE([1], 1, this + foo.this + value + this.value)
    test_algebrize!(
        with_variable_and_field_accesses_of_same_names,
        method = algebrize_expression,
        expression_context = ExpressionContext::default(),
        expected = Ok(mir::Expression::HigherOrderFunction(
            mir::HigherOrderFunctionApplication::Reduce(mir::ReduceExpr {
                array: Box::new(mir::Expression::Array(mir::ArrayExpr {
                    array: vec![mir::Expression::Literal(mir::LiteralValue::Integer(1))]
                })),
                init_value: Box::new(mir::Expression::Literal(mir::LiteralValue::Integer(1))),
                f: Box::new(mir::Expression::ScalarFunction(
                    mir::ScalarFunctionApplication {
                        function: mir::ScalarFunction::Add,
                        args: vec![
                            mir::Expression::Variable(mir::Variable {
                                name: "this".to_string(),
                                is_nullable: false,
                            }),
                            mir::Expression::ScalarFunction(mir::ScalarFunctionApplication {
                                function: mir::ScalarFunction::Add,
                                args: vec![
                                    mir::Expression::FieldAccess(mir::FieldAccess {
                                        expr: Box::new(mir::Expression::Reference(
                                            mir::ReferenceExpr {
                                                key: ("foo", 0u16).into(),
                                            }
                                        )),
                                        field: "this".to_string(),
                                        is_nullable: false,
                                    }),
                                    mir::Expression::ScalarFunction(
                                        mir::ScalarFunctionApplication {
                                            function: mir::ScalarFunction::Add,
                                            args: vec![
                                                mir::Expression::Variable(mir::Variable {
                                                    name: "value".to_string(),
                                                    is_nullable: false,
                                                }),
                                                mir::Expression::FieldAccess(mir::FieldAccess {
                                                    expr: Box::new(mir::Expression::Reference(
                                                        mir::ReferenceExpr {
                                                            key: ("foo", 0u16).into(),
                                                        }
                                                    )),
                                                    field: "value".to_string(),
                                                    is_nullable: false,
                                                }),
                                            ],
                                            is_nullable: false,
                                        }
                                    )
                                ],
                                is_nullable: false,
                            })
                        ],
                        is_nullable: false,
                    }
                )),
                is_nullable: false,
            })
        )),
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Reduce(
            ast::ReduceExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                init_value: Box::new(ast::Expression::Literal(ast::Literal::Integer(1))),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Binary(
                    ast::BinaryExpr {
                        left: Box::new(ast::Expression::Identifier("this".to_string())),
                        op: ast::BinaryOp::Add,
                        right: Box::new(ast::Expression::Binary(ast::BinaryExpr {
                            left: Box::new(ast::Expression::Subpath(ast::SubpathExpr {
                                expr: Box::new(ast::Expression::Identifier("foo".to_string())),
                                subpath: "this".to_string(),
                            })),
                            op: ast::BinaryOp::Add,
                            right: Box::new(ast::Expression::Binary(ast::BinaryExpr {
                                left: Box::new(ast::Expression::Identifier("value".to_string())),
                                op: ast::BinaryOp::Add,
                                right: Box::new(ast::Expression::Subpath(ast::SubpathExpr {
                                    expr: Box::new(ast::Expression::Identifier("foo".to_string())),
                                    subpath: "value".to_string(),
                                })),
                            })),
                        })),
                    }
                ))),
            }
        )),
        env = map! {
            ("foo", 0u16).into() => Schema::Document( Document {
                keys: map! {
                    "this".into() => Schema::Atomic(Atomic::Integer),
                    "value".into() => Schema::Atomic(Atomic::Integer),
                },
                required: set! { "this".to_string(), "value".to_string() },
                additional_properties: false,
                ..Default::default()
            }),
        },
    );

    test_algebrize!(
        invalid_array_arg,
        method = algebrize_expression,
        expected = Err(Error::HigherOrderFunctionWrapper {
            name: "Reduce",
            cause: HigherOrderFunctionErrorCause::ArrayArg,
            error: Box::new(Error::FieldNotFound(
                "foo".to_string(),
                None,
                ClauseType::Unintialized,
                0u16
            )),
        }),
        expected_error_code = 3035,
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Reduce(
            ast::ReduceExpr {
                array: Box::new(ast::Expression::Subpath(ast::SubpathExpr {
                    expr: Box::new(ast::Expression::Identifier("foo".to_string())),
                    subpath: "a".to_string(),
                })),
                init_value: Box::new(ast::Expression::Literal(ast::Literal::Integer(1))),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Literal(
                    ast::Literal::Integer(1)
                ))),
            }
        )),
    );

    test_algebrize!(
        invalid_init_value,
        method = algebrize_expression,
        expected = Err(Error::HigherOrderFunctionWrapper {
            name: "Reduce",
            cause: HigherOrderFunctionErrorCause::InitialValue,
            error: Box::new(Error::FieldNotFound(
                "foo".to_string(),
                None,
                ClauseType::Unintialized,
                0u16
            )),
        }),
        expected_error_code = 3035,
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Reduce(
            ast::ReduceExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                init_value: Box::new(ast::Expression::Subpath(ast::SubpathExpr {
                    expr: Box::new(ast::Expression::Identifier("foo".to_string())),
                    subpath: "a".to_string(),
                })),
                f: Box::new(ast::FunctionArgument::Expr(ast::Expression::Literal(
                    ast::Literal::Integer(1)
                ))),
            }
        )),
    );

    test_algebrize!(
        invalid_function_arg,
        method = algebrize_expression,
        expected = Err(Error::HigherOrderFunctionWrapper {
            name: "Reduce",
            cause: HigherOrderFunctionErrorCause::FunctionArg,
            error: Box::new(Error::FieldNotFound(
                "foo".to_string(),
                None,
                ClauseType::Unintialized,
                0u16
            )),
        }),
        expected_error_code = 3035,
        input = ast::Expression::HigherOrderFunction(ast::HigherOrderFunctionExpr::Reduce(
            ast::ReduceExpr {
                array: Box::new(ast::Expression::Array(vec![ast::Expression::Literal(
                    ast::Literal::Integer(1)
                )])),
                init_value: Box::new(ast::Expression::Literal(ast::Literal::Integer(1))),
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
            }
        )),
    );
}
