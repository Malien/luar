use luar_lex::{fmt_tokens, DynTokens, ToTokenStream, Token};

use crate::{expr::Expression, Block};

#[derive(Debug, PartialEq, Clone)]
pub struct RepeatLoop {
    pub body: Block,
    pub condition: Expression,
}

impl ToTokenStream for RepeatLoop {
    type Tokens = DynTokens;
    fn to_tokens(self) -> Self::Tokens {
        let Self { body, condition } = self;
        Box::new(
            std::iter::once(Token::Repeat)
                .chain(body.to_tokens())
                .chain(std::iter::once(Token::Until))
                .chain(condition.to_tokens()),
        )
    }
}

fmt_tokens!(RepeatLoop);
#[cfg(feature = "quickcheck")]
use quickcheck::{Arbitrary, Gen};

#[cfg(feature = "quickcheck")]
impl Arbitrary for RepeatLoop {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            condition: Arbitrary::arbitrary(g),
            body: Arbitrary::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let condition = self.condition.clone();
        let body = self.body.clone();
        Box::new(
            self.condition
                .shrink()
                .map(move |condition| Self {
                    condition,
                    body: body.clone(),
                })
                .chain(self.body.shrink().map(move |body| Self {
                    condition: condition.clone(),
                    body,
                })),
        )
    }
}

#[cfg(test)]
mod test {
    use luar_lex::{Ident, NumberLiteral};
    use non_empty::NonEmptyVec;

    use crate::{expr::Expression, input_parsing_expectation, Block, Declaration, Statement};

    #[cfg(feature = "quickcheck")]
    use crate::unspanned_lua_token_parser;
    #[cfg(feature = "quickcheck")]
    use luar_lex::ToTokenStream;

    use super::RepeatLoop;

    #[test]
    fn correctly_displays() {
        let repeat_loop = RepeatLoop {
            condition: Expression::Nil,
            body: Block {
                statements: vec![
                    Statement::LocalDeclaration(Declaration {
                        names: NonEmptyVec::of_single(Ident::new("foo")),
                        initial_values: vec![Expression::Number(NumberLiteral(42f64))],
                    }),
                    Statement::LocalDeclaration(Declaration {
                        names: NonEmptyVec::of_single(Ident::new("bar")),
                        initial_values: vec![Expression::Number(NumberLiteral(69f64))],
                    }),
                ],
                ret: None,
            },
        };
        assert_eq!(
            "repeat\n\tlocal foo = 42\n\tlocal bar = 69\nuntil nil",
            format!("{}", repeat_loop)
        );
    }

    input_parsing_expectation!(
        repeat_loop,
        parses_empty,
        "repeat until 1",
        RepeatLoop {
            body: Block {
                statements: vec![],
                ret: None
            },
            condition: Expression::Number(NumberLiteral(1f64))
        }
    );

    input_parsing_expectation!(
        repeat_loop,
        parses_single_statement,
        "repeat 
            local foo = 42
        until 1",
        RepeatLoop {
            body: Block {
                statements: vec![Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(Ident::new("foo")),
                    initial_values: vec![Expression::Number(NumberLiteral(42f64))]
                })],
                ret: None
            },
            condition: Expression::Number(NumberLiteral(1f64))
        }
    );

    input_parsing_expectation!(
        repeat_loop,
        parses_multiple_statement_body,
        "repeat
            local foo = 42
            local bar = 69
        until 1",
        RepeatLoop {
            body: Block {
                statements: vec![
                    Statement::LocalDeclaration(Declaration {
                        names: NonEmptyVec::of_single(Ident::new("foo")),
                        initial_values: vec![Expression::Number(NumberLiteral(42f64))]
                    }),
                    Statement::LocalDeclaration(Declaration {
                        names: NonEmptyVec::of_single(Ident::new("bar")),
                        initial_values: vec![Expression::Number(NumberLiteral(69f64))]
                    })
                ],
                ret: None
            },
            condition: Expression::Number(NumberLiteral(1f64))
        }
    );

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn parses_empty_loop_with_arbitrary_condition(condition: Expression) {
        let repeat_loop = RepeatLoop {
            body: Block {
                statements: vec![],
                ret: None,
            },
            condition,
        };
        let tokens: Vec<_> = repeat_loop.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::repeat_loop(tokens).unwrap();
        assert_eq!(repeat_loop, parsed)
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn parses_loop_with_arbitrary_body(body: Block) {
        let repeat_loop = RepeatLoop {
            body,
            condition: Expression::Nil,
        };
        let tokens: Vec<_> = repeat_loop.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::repeat_loop(tokens).unwrap();
        assert_eq!(repeat_loop, parsed);
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn parses_arbitrary_repeat_loop(repeat_loop: RepeatLoop) {
        let tokens: Vec<_> = repeat_loop.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::repeat_loop(tokens).unwrap();
        assert_eq!(repeat_loop, parsed);
    }
}
