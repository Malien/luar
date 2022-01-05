use super::{ArithmeticError, EvalContext, EvalError, LuaValue, TypeError};
use crate::{
    lex::{NumberLiteral, StringLiteral},
    syn::{self, Assignment},
};

pub trait Eval {
    type Return;

    fn eval(&self, context: &mut impl EvalContext) -> Result<Self::Return, EvalError>;
}

impl<T: Eval> Eval for Box<T> {
    type Return = T::Return;
    fn eval(&self, context: &mut impl EvalContext) -> Result<Self::Return, EvalError> {
        T::eval(self, context)
    }
}

impl Eval for syn::Module {
    type Return = LuaValue;

    fn eval(&self, context: &mut impl EvalContext) -> Result<LuaValue, EvalError> {
        for chunk in &*self.chunks {
            chunk.eval(context)?;
        }
        match self.ret {
            Some(ref ret) => ret.eval(context),
            None => Ok(LuaValue::Nil),
        }
    }
}

impl Eval for syn::Chunk {
    type Return = ();

    fn eval(&self, context: &mut impl EvalContext) -> Result<(), EvalError> {
        match self {
            syn::Chunk::Statement(stmnt) => stmnt.eval(context),
            syn::Chunk::FnDecl(decl) => decl.eval(context),
        }
    }
}

impl Eval for syn::Return {
    type Return = LuaValue;

    fn eval(&self, context: &mut impl EvalContext) -> Result<Self::Return, EvalError> {
        match self.0 {
            Some(ref expr) => expr.eval(context),
            None => Ok(LuaValue::Nil),
        }
    }
}

impl Eval for syn::Expression {
    type Return = LuaValue;

    fn eval(&self, context: &mut impl EvalContext) -> Result<Self::Return, EvalError> {
        match self {
            syn::Expression::Nil => Ok(LuaValue::Nil),
            syn::Expression::Number(NumberLiteral(num)) => Ok(LuaValue::Number(*num)),
            syn::Expression::String(StringLiteral(str)) => Ok(LuaValue::String(str.clone())),
            syn::Expression::Variable(var) => var.eval(context),
            syn::Expression::TableConstructor(tbl) => tbl.eval(context),
            syn::Expression::FunctionCall(call) => call.eval(context),
            syn::Expression::UnaryOperator { op, exp } => {
                exp.as_ref().eval(context).and_then(|val| {
                    unary_op_eval(*op, val)
                        .map_err(|err| EvalError::TypeError(TypeError::Arithmetic(err)))
                })
            }
            syn::Expression::BinaryOperator { lhs, op, rhs } => {
                binary_op_eval(*op, lhs, rhs, context)
            }
        }
    }
}

fn unary_op_eval(op: syn::UnaryOperator, value: LuaValue) -> Result<LuaValue, ArithmeticError> {
    match op {
        syn::UnaryOperator::Minus => unary_minus_eval(value),
        syn::UnaryOperator::Not if value.is_falsy() => Ok(LuaValue::Number(1f64)),
        syn::UnaryOperator::Not => Ok(LuaValue::Nil),
    }
}

fn unary_minus_eval(value: LuaValue) -> Result<LuaValue, ArithmeticError> {
    match value {
        LuaValue::Number(num) => Ok(LuaValue::Number(-num)),
        LuaValue::String(str) => str_unary_minus(str),
        value => Err(ArithmeticError::UnaryMinus(value)),
    }
}

fn str_unary_minus(str: String) -> Result<LuaValue, ArithmeticError> {
    str.parse::<f64>()
        .map(|num| LuaValue::Number(-num))
        .map_err(|_| ArithmeticError::UnaryMinus(LuaValue::String(str)))
}

fn binary_op_eval(
    op: syn::BinaryOperator,
    lhs: &impl Eval<Return = LuaValue>,
    rhs: &impl Eval<Return = LuaValue>,
    context: &mut impl EvalContext,
) -> Result<LuaValue, EvalError> {
    use syn::BinaryOperator::*;

    let lhs = lhs.eval(context)?;
    match op {
        And if lhs.is_falsy() => Ok(lhs),
        And => rhs.eval(context),
        Or if lhs.is_truthy() => Ok(lhs),
        Or => rhs.eval(context),
        // // Precedence level 1
        // Less,
        // Greater,
        // LessOrEquals,
        // GreaterOrEquals,
        // NotEquals,
        // Equals,
        // // Precedence level 2
        // Concat,
        // // Precedence level 3
        // Plus,
        // Minus,
        // // Precedence level 4
        // Mul,
        // Div,
        // // Precedence level 5
        // Exp,
        _ => todo!(),
    }
}

// fn binary_and_eval(
//     lhs: LuaValue,
//     rhs: &impl Eval<Return = LuaValue>,
//     context: &mut EvalContext,
// ) -> Result<LuaValue, EvalError> {
//     if lhs.is_falsy() {
//         Ok(lhs)
//     } else {
//         rhs.eval(context)
//     }
// }

impl Eval for syn::Var {
    type Return = LuaValue;

    fn eval(&self, context: &mut impl EvalContext) -> Result<Self::Return, EvalError> {
        match self {
            Self::Named(ident) => Ok(context.get(ident).clone()),
            _ => todo!(),
        }
    }
}

impl Eval for syn::Statement {
    type Return = ();

    fn eval(&self, context: &mut impl EvalContext) -> Result<Self::Return, EvalError> {
        match self {
            Self::Assignment(Assignment { names, values }) => {
                let (stored, throwaway) = values.split_at(names.len());
                let stored = stored.into_iter().map(|expr| expr.eval(context)).collect::<Result<Vec<_>, _>>()?;
                for expr in throwaway {
                    expr.eval(context)?;
                }
                for (var, value) in names.into_iter().zip(stored) {
                    assign_to_var(context, var, value);
                }
                Ok(())
            }
            _ => todo!(),
        }
    }
}

fn assign_to_var(context: &mut impl EvalContext, var: &syn::Var, value: LuaValue) {
    match var {
        syn::Var::Named(ident) => context.set(ident.clone(), value),
        _ => todo!(),
    }
}

macro_rules! todo_eval {
    ($ret: ty, $name: ty) => {
        impl Eval for $name {
            type Return = $ret;

            fn eval(&self, _: &mut impl EvalContext) -> Result<Self::Return, EvalError> {
                todo!();
            }
        }
    };
}

// todo_eval!(LuaValue, syn::Var);
todo_eval!(LuaValue, syn::TableConstructor);
todo_eval!(LuaValue, syn::FunctionCall);
// todo_eval!((), syn::Statement);
todo_eval!((), syn::FunctionDeclaration);

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use itertools::Itertools;
    use quickcheck::TestResult;

    use super::Eval;
    use crate::error::LuaError;
    use crate::lang::{EvalContext, GlobalContext, LuaValue};
    use crate::lex::{Ident, NumberLiteral, StringLiteral, Token};
    use crate::syn;
    use crate::test_util::Finite;
    use crate::util::NonEmptyVec;

    mod expressions {
        use quickcheck::Arbitrary;

        use crate::{
            error::LuaError,
            lang::{ArithmeticError, Eval, EvalError, GlobalContext, LuaValue, TypeError},
            lex::{NumberLiteral, StringLiteral},
            syn,
            test_util::Finite,
            util::eq_with_nan,
        };

        #[test]
        fn eval_nil() -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = syn::Expression::Nil;
            assert_eq!(expr.eval(&mut context)?, LuaValue::Nil);
            Ok(())
        }

        #[quickcheck]
        fn eval_number_literal(num: f64) -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = syn::Expression::Number(NumberLiteral(num));
            assert!(eq_with_nan(expr.eval(&mut context)?.unwrap_number(), num));
            Ok(())
        }

        #[quickcheck]
        fn eval_string_literal(str: String) -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = syn::Expression::String(StringLiteral(str.clone()));
            assert_eq!(expr.eval(&mut context)?.unwrap_string(), str);
            Ok(())
        }

        fn negation_expr(num: f64) -> syn::Expression {
            syn::Expression::UnaryOperator {
                op: syn::UnaryOperator::Minus,
                exp: Box::new(syn::Expression::Number(NumberLiteral(num))),
            }
        }

        #[quickcheck]
        fn eval_negation(Finite(num): Finite<f64>) -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = negation_expr(num);
            assert_eq!(expr.eval(&mut context)?.unwrap_number(), -num);
            Ok(())
        }

        #[test]
        fn eval_negation_on_nan() -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = negation_expr(f64::NAN);
            assert!(eq_with_nan(
                expr.eval(&mut context)?.unwrap_number(),
                f64::NAN
            ));
            Ok(())
        }

        #[test]
        fn eval_negation_on_inf() -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = negation_expr(f64::INFINITY);

            assert!(eq_with_nan(
                expr.eval(&mut context)?.unwrap_number(),
                f64::NEG_INFINITY
            ));
            Ok(())
        }

        #[test]
        fn eval_negation_on_neg_inf() -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = negation_expr(f64::NEG_INFINITY);

            assert!(eq_with_nan(
                expr.eval(&mut context)?.unwrap_number(),
                f64::INFINITY
            ));
            Ok(())
        }

        #[quickcheck]
        fn eval_negation_on_convertible_str(num: f64) -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = syn::Expression::UnaryOperator {
                op: syn::UnaryOperator::Minus,
                exp: Box::new(syn::Expression::String(StringLiteral(format!("{}", num)))),
            };

            assert!(eq_with_nan(expr.eval(&mut context)?.unwrap_number(), -num));
            Ok(())
        }

        #[test]
        fn eval_unary_minus_on_unsupported_type_errors() {
            let mut context = GlobalContext::new();
            let unsupported = [
                syn::Expression::Nil,
                syn::Expression::String(StringLiteral("Definitely not a number".to_string())),
                // syn::Expression::TableConstructor(TableConstructor::empty()),
            ];

            for exp in unsupported {
                let expr = syn::Expression::UnaryOperator {
                    op: syn::UnaryOperator::Minus,
                    exp: Box::new(exp),
                };
                let res = expr.eval(&mut context);
                println!("{:?}", res);
                assert!(matches!(
                    res,
                    Err(EvalError::TypeError(TypeError::Arithmetic(
                        ArithmeticError::UnaryMinus(_)
                    )))
                ));
            }
        }

        #[test]
        fn eval_not_on_nil() -> Result<(), LuaError> {
            let mut context = GlobalContext::new();
            let exp = syn::string_parser::expression("not nil")?;
            assert_eq!(exp.eval(&mut context)?, LuaValue::Number(1f64));
            Ok(())
        }

        #[derive(Debug, Clone)]
        pub struct TruthyExpression(syn::Expression);

        impl Arbitrary for TruthyExpression {
            fn arbitrary(g: &mut quickcheck::Gen) -> Self {
                Self(match u8::arbitrary(g) % 2 {
                    0 => syn::Expression::Number(Arbitrary::arbitrary(g)),
                    1 => syn::Expression::String(Arbitrary::arbitrary(g)),
                    _ => unreachable!(),
                })
            }

            fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
                Box::new(self.0.shrink().map(Self))
            }
        }

        #[quickcheck]
        fn eval_not_on_truthy_vals(
            TruthyExpression(expr): TruthyExpression,
        ) -> Result<(), LuaError> {
            let mut context = GlobalContext::new();
            let expr = syn::Expression::UnaryOperator {
                op: syn::UnaryOperator::Not,
                exp: Box::new(expr),
            };
            assert_eq!(expr.eval(&mut context)?, LuaValue::Nil);
            Ok(())
        }

        #[derive(Debug, Clone)]
        pub struct SimpleExpression(syn::Expression);

        impl Arbitrary for SimpleExpression {
            fn arbitrary(g: &mut quickcheck::Gen) -> Self {
                Self(match u8::arbitrary(g) % 3 {
                    0 => syn::Expression::Nil,
                    1 => syn::Expression::Number(Arbitrary::arbitrary(g)),
                    2 => syn::Expression::String(Arbitrary::arbitrary(g)),
                    _ => unreachable!(),
                })
            }

            fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
                Box::new(self.0.shrink().map(Self))
            }
        }

        #[quickcheck]
        fn eval_and_on_truthy(
            TruthyExpression(lhs): TruthyExpression,
            SimpleExpression(rhs): SimpleExpression,
        ) -> Result<(), LuaError> {
            let mut context = GlobalContext::new();
            let expected = rhs.eval(&mut context)?;
            let expr = syn::Expression::BinaryOperator {
                op: syn::BinaryOperator::And,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };

            assert!(expr.eval(&mut context)?.total_eq(&expected));
            Ok(())
        }

        #[quickcheck]
        fn eval_and_on_falsy(SimpleExpression(rhs): SimpleExpression) -> Result<(), LuaError> {
            let mut context = GlobalContext::new();
            let expr = syn::Expression::BinaryOperator {
                op: syn::BinaryOperator::And,
                lhs: Box::new(syn::Expression::Nil),
                rhs: Box::new(rhs),
            };

            assert_eq!(expr.eval(&mut context)?, LuaValue::Nil);
            Ok(())
        }
    }

    #[quickcheck]
    fn eval_ident_on_global(value: LuaValue, ident: Ident) -> Result<(), LuaError> {
        let module = syn::string_parser::module(&format!("return {}", ident))?;
        let mut context = GlobalContext::new();
        assert_eq!(module.eval(&mut context)?, LuaValue::Nil);
        context.set(ident, value.clone());
        assert!(module.eval(&mut context)?.total_eq(&value));
        Ok(())
    }

    #[test]
    fn eval_nil() -> Result<(), LuaError> {
        let module = syn::string_parser::module("return nil")?;
        let mut context = GlobalContext::new();
        assert_eq!(module.eval(&mut context)?, LuaValue::Nil);
        Ok(())
    }

    #[quickcheck]
    fn eval_number_literal(Finite(num): Finite<f64>) -> Result<(), LuaError> {
        let module = syn::lua_parser::module(&[Token::Return, Token::Number(NumberLiteral(num))])?;
        let mut context = GlobalContext::new();
        assert!(module.eval(&mut context)?.total_eq(&LuaValue::Number(num)));
        Ok(())
    }

    #[quickcheck]
    fn eval_string_literal(str: String) -> Result<(), LuaError> {
        let module =
            syn::lua_parser::module(&[Token::Return, Token::String(StringLiteral(str.clone()))])?;
        let mut context = GlobalContext::new();
        assert_eq!(module.eval(&mut context)?, LuaValue::String(str));
        Ok(())
    }

    #[quickcheck]
    fn eval_and_truthy(lhs: LuaValue, rhs: LuaValue) -> Result<TestResult, LuaError> {
        if lhs.is_falsy() {
            return Ok(TestResult::discard());
        }
        let module = syn::string_parser::module("return lhs and rhs")?;
        let mut context = GlobalContext::new();
        context.set("lhs", lhs);
        context.set("rhs", rhs.clone());
        let res = module.eval(&mut context)?;
        assert!(LuaValue::total_eq(&res, &rhs));
        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn eval_and_falsy(rhs: LuaValue) -> Result<TestResult, LuaError> {
        let module = syn::string_parser::module("return lhs and rhs")?;
        let mut context = GlobalContext::new();
        context.set("lhs", LuaValue::Nil);
        context.set("rhs", rhs.clone());
        let res = module.eval(&mut context)?;
        assert_eq!(res, LuaValue::Nil);
        Ok(TestResult::passed())
    }

    #[test]
    #[ignore = "This relies on fn def, fn call and assignment which are not implemented yet"]
    fn and_short_circuits() -> Result<(), LuaError> {
        let module = syn::string_parser::module(
            "side_effect_committed = nil

            function side_effecty_fn()
                side_effect_committed = 1
            end

            res = nil and side_effecty_fn()
            return side_effect_committed",
        )?;
        let mut context = GlobalContext::new();
        assert_eq!(module.eval(&mut context)?, LuaValue::Nil);
        Ok(())
    }

    #[quickcheck]
    fn eval_or_truthy(lhs: LuaValue, rhs: LuaValue) -> Result<TestResult, LuaError> {
        if lhs.is_falsy() {
            return Ok(TestResult::discard());
        }
        let module = syn::string_parser::module("return lhs or rhs")?;
        println!("{:#?}", module);
        let mut context = GlobalContext::new();
        context.set("lhs", lhs.clone());
        context.set("rhs", rhs);
        let res = module.eval(&mut context)?;
        assert!(LuaValue::total_eq(&res, &lhs));
        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn eval_or_falsy(rhs: LuaValue) -> Result<TestResult, LuaError> {
        let module = syn::string_parser::module("return lhs or rhs")?;
        let mut context = GlobalContext::new();
        context.set("lhs", LuaValue::Nil);
        context.set("rhs", rhs.clone());
        let res = module.eval(&mut context)?;
        assert!(LuaValue::total_eq(&res, &rhs));
        Ok(TestResult::passed())
    }

    #[test]
    #[ignore = "This relies on fn def, fn call and assignment which are not implemented yet"]
    fn or_short_circuits() -> Result<(), LuaError> {
        let module = syn::string_parser::module(
            "
            side_effect_committed = nil

            function side_effecty_fn()
                side_effect_committed = 1
            end

            res = 1 or side_effecty_fn()
            return side_effect_committed
        ",
        )?;
        let mut context = GlobalContext::new();
        assert_eq!(module.eval(&mut context)?, LuaValue::Nil);
        Ok(())
    }

    #[quickcheck]
    fn eval_single_assignment(ident: Ident, v1: LuaValue, v2: LuaValue) -> Result<(), LuaError> {
        let module = syn::lua_parser::module(&[
            Token::Ident(ident.clone()),
            Token::Assignment,
            Token::Ident(Ident::new("value")),
        ])?;
        let mut context = GlobalContext::new();
        assert_eq!(context.get(&ident), &LuaValue::Nil);
        context.set("value", v1.clone());
        module.eval(&mut context)?;
        assert!(context.get(&ident).total_eq(&v1));
        context.set("value", v2.clone());
        module.eval(&mut context)?;
        assert!(context.get(&ident).total_eq(&v2));
        Ok(())
    }

    #[quickcheck]
    fn eval_multiple_assignment(
        idents: HashSet<Ident>,
        values: NonEmptyVec<LuaValue>,
    ) -> Result<TestResult, LuaError> {
        if idents.len() == 0 {
            return Ok(TestResult::discard());
        }
        // Make iteration order deterministic
        let idents: Vec<_> = idents.into_iter().collect();
        let value_idents: Vec<_> = (0..values.len())
            .into_iter()
            .map(|i| format!("value{}", i))
            .map(Ident::new)
            .collect();
        let tokens: Vec<_> = idents
            .iter()
            .cloned()
            .map(Token::Ident)
            .intersperse_with(|| Token::Comma)
            .chain(std::iter::once(Token::Assignment))
            .chain(
                value_idents
                    .iter()
                    .cloned()
                    .map(Token::Ident)
                    .intersperse_with(|| Token::Comma),
            )
            .collect();
        let module = syn::lua_parser::module(&tokens)?;
        let mut context = GlobalContext::new();
        for (ident, value) in value_idents.iter().cloned().zip(values.iter().cloned()) {
            context.set(ident, value);
        }
        module.eval(&mut context)?;

        if idents.len() > values.len() {
            for ident in &idents[values.len()..] {
                assert_eq!(context.get(ident), &LuaValue::Nil);
            }
        }

        for (ident, value) in idents.into_iter().zip(values) {
            assert!(context.get(&ident).total_eq(&value));
        }

        Ok(TestResult::passed())
    }
}
