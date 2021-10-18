use std::iter;

use crate::lex::{DynTokens, NumberLiteral, StringLiteral, ToTokenStream, Token};

pub mod op;
pub mod table_constructor;
pub mod var;

pub use self::{op::*, table_constructor::*, var::*};

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Nil,
    String(StringLiteral),
    Number(NumberLiteral),
    Variable(Var),
    BinaryOperator {
        lhs: Box<Expression>,
        op: BinaryOperator,
        rhs: Box<Expression>,
    },
    UnaryOperator {
        op: UnaryOperator,
        exp: Box<Expression>,
    },
    TableConstructor(TableConstructor),
    FunctionCall {
        func: Var,
        args: Vec<Expression>,
    },
}

impl ToTokenStream for Expression {
    type Tokens = DynTokens;
    fn to_tokens(self) -> Self::Tokens {
        match self {
            Expression::Nil => Box::new(iter::once(Token::Nil)),
            Expression::String(literal) => Box::new(literal.to_tokens()),
            Expression::Number(literal) => Box::new(literal.to_tokens()),
            Expression::Variable(var) => var.to_tokens(),
            Expression::BinaryOperator { lhs, op, rhs } => Box::new(
                iter::once(Token::OpenRoundBracket)
                    .chain(lhs.to_tokens())
                    .chain(op.to_tokens())
                    .chain(rhs.to_tokens())
                    .chain(iter::once(Token::CloseRoundBracket)),
            ),
            Expression::UnaryOperator { op, exp } => Box::new(
                iter::once(Token::OpenRoundBracket)
                    .chain(op.to_tokens())
                    .chain(exp.to_tokens())
                    .chain(iter::once(Token::CloseRoundBracket)),
            ),
            Expression::TableConstructor(constructor) => constructor.to_tokens(),
            Expression::FunctionCall { .. } => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        syn::expr::TableConstructor,
        test_util::{with_thread_gen, QUICKCHECK_RECURSIVE_DEPTH},
    };
    use quickcheck::{empty_shrinker, Arbitrary, Gen, TestResult};
    use std::iter;

    use crate::{
        lex::{NumberLiteral, StringLiteral, ToTokenStream, Token},
        syn::{expr::Var, lua_parser, BinaryOperator, UnaryOperator},
    };

    use super::Expression;

    impl Arbitrary for Expression {
        fn arbitrary(g: &mut Gen) -> Self {
            if g.size() == 0 {
                with_thread_gen(|gen| match u8::arbitrary(gen) % 3 {
                    0 => Expression::Nil,
                    1 => Expression::Number(NumberLiteral::arbitrary(gen)),
                    2 => Expression::String(StringLiteral::arbitrary(gen)),
                    _ => unreachable!(),
                })
            } else {
                let mut gen = Gen::new(QUICKCHECK_RECURSIVE_DEPTH.min(g.size() - 1));
                let g = &mut gen;
                match u8::arbitrary(g) % 4 {
                    0 => Expression::Variable(Var::arbitrary(g)),
                    1 => Expression::UnaryOperator {
                        op: UnaryOperator::arbitrary(g),
                        exp: Box::new(Expression::arbitrary(g)),
                    },
                    2 => Expression::BinaryOperator {
                        op: BinaryOperator::arbitrary(g),
                        lhs: Box::new(Expression::arbitrary(g)),
                        rhs: Box::new(Expression::arbitrary(g)),
                    },
                    3 => Expression::TableConstructor(TableConstructor::arbitrary(g)),
                    _ => unreachable!(),
                }
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match self {
                // Expression::Nil => empty_shrinker(),
                // Expression::Number(_) => Box::new(iter::once(Expression::Number(NumberLiteral(42.0)))),
                // Expression::String(_) => Box::new(iter::once(Expression::String(StringLiteral("str".to_string())))),
                // Expression::Number(_) | Expression::String(_) => Box::new(iter::once(Expression::Nil)),
                Expression::Variable(var) => Box::new(var.shrink().map(Expression::Variable)),
                Expression::UnaryOperator { exp, .. } => Box::new(iter::once(exp.as_ref().clone())),
                Expression::BinaryOperator { lhs, rhs, .. } => Box::new(
                    iter::once(lhs.as_ref().clone()).chain(iter::once(rhs.as_ref().clone())),
                ),
                Expression::TableConstructor(tbl) => {
                    Box::new(tbl.shrink().map(Expression::TableConstructor))
                }
                // Expression::FunctionCall { .. } => empty_shrinker(),
                // Expression::FunctionCall { args } => Box::new(args.map(Expression::shrink).chain(iter::once(Expression::FunctionCall)))
                Expression::Nil
                | Expression::Number(_)
                | Expression::String(_)
                | Expression::FunctionCall { .. } => empty_shrinker(),
            }
        }
    }

    #[test]
    fn nill_expr() {
        let parsed = lua_parser::expression(&[Token::Nil]).unwrap();
        assert_eq!(Expression::Nil, parsed);
    }

    #[quickcheck]
    fn number_expr(literal: NumberLiteral) {
        let expression = lua_parser::expression(&[Token::Number(literal)]).unwrap();
        match (&literal, &expression) {
            (NumberLiteral(x), Expression::Number(NumberLiteral(y))) if f64::is_nan(*x) => {
                assert!(f64::is_nan(*y))
            }
            _ => assert_eq!(Expression::Number(literal), expression),
        };
    }

    #[quickcheck]
    fn string_expr(literal: StringLiteral) {
        assert_eq!(
            Expression::String(literal.clone()),
            lua_parser::expression(&[Token::String(literal)]).unwrap()
        );
    }

    #[quickcheck]
    #[ignore]
    fn var_expr(expected: Var) {
        let tokens = expected.clone().to_tokens().collect::<Vec<_>>();
        let parsed = lua_parser::expression(&tokens).unwrap();
        assert_eq!(parsed, Expression::Variable(expected));
    }

    #[quickcheck]
    #[ignore]
    fn parses_arbitrary_expression(expected: Expression) {
        let tokens = expected.clone().to_tokens().collect::<Vec<_>>();
        let parsed = lua_parser::expression(&tokens).unwrap();
        assert_eq!(parsed, expected);
    }
}
