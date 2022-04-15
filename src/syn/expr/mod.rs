use std::iter;

use luar_lex::{fmt_tokens, DynTokens, NumberLiteral, StringLiteral, ToTokenStream, Token};

pub mod function_call;
pub mod op;
pub mod table_constructor;
pub mod var;

pub use self::{function_call::*, op::*, table_constructor::*, var::*};

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
    FunctionCall(FunctionCall),
}

impl ToTokenStream for Expression {
    type Tokens = DynTokens;
    fn to_tokens(self) -> Self::Tokens {
        use Expression::*;
        match self {
            Nil => Box::new(iter::once(Token::Nil)),
            String(literal) => Box::new(literal.to_tokens()),
            Number(literal) => Box::new(literal.to_tokens()),
            Variable(var) => var.to_tokens(),
            BinaryOperator { lhs, op, rhs } => Box::new(
                iter::once(Token::OpenRoundBracket)
                    .chain(lhs.to_tokens())
                    .chain(op.to_tokens())
                    .chain(rhs.to_tokens())
                    .chain(iter::once(Token::CloseRoundBracket)),
            ),
            UnaryOperator { op, exp } => Box::new(
                iter::once(Token::OpenRoundBracket)
                    .chain(op.to_tokens())
                    .chain(exp.to_tokens())
                    .chain(iter::once(Token::CloseRoundBracket)),
            ),
            TableConstructor(constructor) => constructor.to_tokens(),
            FunctionCall(func) => func.to_tokens(),
        }
    }
}

fmt_tokens!(Expression);

#[cfg(test)]
mod test {
    use logos::Logos;
    use luar_lex::{NumberLiteral, StringLiteral, ToTokenStream, Token};
    use quickcheck::{empty_shrinker, Arbitrary, Gen};
    use std::iter;

    use crate::{
        syn::{
            expr::{FunctionCall, TableConstructor, Var},
            unspanned_lua_token_parser, BinaryOperator, UnaryOperator,
        },
        test_util::{with_thread_gen, QUICKCHECK_RECURSIVE_DEPTH},
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
                match u8::arbitrary(g) % 5 {
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
                    4 => Expression::FunctionCall(FunctionCall::arbitrary(g)),
                    _ => unreachable!(),
                }
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            use Expression::*;
            match self {
                // Expression::Nil => empty_shrinker(),
                // Expression::Number(_) => Box::new(iter::once(Expression::Number(NumberLiteral(42.0)))),
                // Expression::String(_) => Box::new(iter::once(Expression::String(StringLiteral("str".to_string())))),
                // Expression::Number(_) | Expression::String(_) => Box::new(iter::once(Expression::Nil)),
                Variable(var) => Box::new(var.shrink().map(Variable)),
                UnaryOperator { exp, .. } => Box::new(iter::once(exp.as_ref().clone())),
                BinaryOperator { lhs, rhs, .. } => Box::new(
                    iter::once(lhs.as_ref().clone()).chain(iter::once(rhs.as_ref().clone())),
                ),
                TableConstructor(tbl) => Box::new(tbl.shrink().map(TableConstructor)),
                FunctionCall(func) => Box::new(func.shrink().map(Expression::FunctionCall)),
                // Expression::FunctionCall { .. } => empty_shrinker(),
                // Expression::FunctionCall { args } => Box::new(args.map(Expression::shrink).chain(iter::once(Expression::FunctionCall)))
                Nil | Number(_) | String(_) => empty_shrinker(),
            }
        }
    }

    #[test]
    fn nill_expr() {
        let parsed = unspanned_lua_token_parser::expression([Token::Nil]).unwrap();
        assert_eq!(Expression::Nil, parsed);
    }

    #[quickcheck]
    fn number_expr(literal: NumberLiteral) {
        let expression = unspanned_lua_token_parser::expression([Token::Number(literal)]).unwrap();
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
            unspanned_lua_token_parser::expression([Token::String(literal)]).unwrap()
        );
    }

    #[quickcheck]
    fn var_expr(expected: Var) {
        let tokens = expected.clone().to_tokens().collect::<Vec<_>>();
        let parsed = unspanned_lua_token_parser::expression(tokens).unwrap();
        assert_eq!(parsed, Expression::Variable(expected));
    }

    #[quickcheck]
    fn parses_arbitrary_expression(expected: Expression) {
        let tokens = expected.clone().to_tokens().collect::<Vec<_>>();
        let parsed = unspanned_lua_token_parser::expression(tokens).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parses_my_example() {
        let tokens: Vec<_> = Token::lexer("A[{}]").collect();
        let parsed = unspanned_lua_token_parser::expression(tokens).unwrap();
        assert_eq!(
            Expression::Variable(Var::MemberLookup {
                from: Box::new(Var::Named("A".parse().unwrap())),
                value: Box::new(Expression::TableConstructor(TableConstructor::empty()))
            }),
            parsed
        );
    }
}
