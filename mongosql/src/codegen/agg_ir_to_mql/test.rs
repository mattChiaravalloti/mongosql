macro_rules! test_codegen_agg_ir_expr {
    ($func_name:ident, expected = $expected:expr, input = $input:expr) => {
        #[test]
        fn $func_name() {
            use crate::codegen::agg_ir_to_mql::MqlCodeGenerator;
            let expected = $expected;
            let input = $input;

            let gen = MqlCodeGenerator { scope_level: 0u16 };
            assert_eq!(expected, gen.codegen_agg_ir_expression(input));
        }
    };
}

mod agg_ir_literal {
    use crate::agg_ir::{Expression::*, LiteralValue::*};
    use bson::{bson, Bson};

    test_codegen_agg_ir_expr!(
        null,
        expected = Ok(bson!({ "$literal": Bson::Null })),
        input = Literal(Null)
    );

    test_codegen_agg_ir_expr!(
        boolean,
        expected = Ok(bson!({ "$literal": Bson::Boolean(true)})),
        input = Literal(Boolean(true))
    );

    test_codegen_agg_ir_expr!(
        string,
        expected = Ok(bson!({ "$literal": Bson::String("foo".to_string())})),
        input = Literal(String("foo".to_string()))
    );

    test_codegen_agg_ir_expr!(
        int,
        expected = Ok(bson!({ "$literal": 1_i32})),
        input = Literal(Integer(1))
    );

    test_codegen_agg_ir_expr!(
        long,
        expected = Ok(bson!({ "$literal": 2_i64})),
        input = Literal(Long(2))
    );

    test_codegen_agg_ir_expr!(
        double,
        expected = Ok(bson!({ "$literal": 3.0})),
        input = Literal(Double(3.0))
    );
}
