use crate::{
    fmt_tokens,
    lex::{DynTokens, ToTokenStream, Token},
    syn::expr::Expression,
};

use super::Statement;

#[derive(Debug, PartialEq, Clone)]
pub struct RepeatLoop {
    pub body: Vec<Statement>,
    pub condition: Expression,
}

impl ToTokenStream for RepeatLoop {
    type Tokens = DynTokens;
    fn to_tokens(self) -> Self::Tokens {
        let Self { body, condition } = self;
        Box::new(
            std::iter::once(Token::Repeat)
                .chain(body.into_iter().flat_map(ToTokenStream::to_tokens))
                .chain(std::iter::once(Token::Until))
                .chain(condition.to_tokens()),
        )
    }
}

fmt_tokens!(RepeatLoop);

#[cfg(test)]
mod test {
    use quickcheck::Arbitrary;

    use crate::{
        input_parsing_expectation,
        lex::{Ident, NumberLiteral, ToTokenStream},
        syn::{expr::Expression, lua_parser, Declaration, Statement},
        util::NonEmptyVec, test_util::GenExt,
    };

    use super::RepeatLoop;

    impl Arbitrary for RepeatLoop {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            if g.size() <= 1 {
                return Self {
                    condition: Expression::Nil,
                    body: vec![],
                };
            }
            let mut inner_gen = g.next_iter();
            let body = std::iter::repeat_with(|| Arbitrary::arbitrary(&mut inner_gen))
                .take(g.size())
                .collect();
            Self {
                condition: Arbitrary::arbitrary(g),
                body,
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

    #[test]
    fn correctly_displays() {
        let repeat_loop = RepeatLoop {
            condition: Expression::Nil,
            body: vec![
                Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(Ident::new("foo")),
                    initial_values: vec![Expression::Number(NumberLiteral(42f64))],
                }),
                Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(Ident::new("bar")),
                    initial_values: vec![Expression::Number(NumberLiteral(69f64))],
                }),
            ],
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
            body: vec![],
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
            body: vec![Statement::LocalDeclaration(Declaration {
                names: NonEmptyVec::of_single(Ident::new("foo")),
                initial_values: vec![Expression::Number(NumberLiteral(42f64))]
            })],
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
            body: vec![
                Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(Ident::new("foo")),
                    initial_values: vec![Expression::Number(NumberLiteral(42f64))]
                }),
                Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(Ident::new("bar")),
                    initial_values: vec![Expression::Number(NumberLiteral(69f64))]
                })
            ],
            condition: Expression::Number(NumberLiteral(1f64))
        }
    );

    #[quickcheck]
    fn parses_empty_loop_with_arbitrary_condition(condition: Expression) {
        let repeat_loop = RepeatLoop {
            body: vec![],
            condition,
        };
        let tokens: Vec<_> = repeat_loop.clone().to_tokens().collect();
        let parsed = lua_parser::repeat_loop(&tokens).unwrap();
        assert_eq!(repeat_loop, parsed)
    }

    #[quickcheck]
    fn parses_loop_with_arbitrary_body(body: Vec<Statement>) {
        let repeat_loop = RepeatLoop {
            body,
            condition: Expression::Nil,
        };
        let tokens: Vec<_> = repeat_loop.clone().to_tokens().collect();
        let parsed = lua_parser::repeat_loop(&tokens).unwrap();
        assert_eq!(repeat_loop, parsed);
    }

    #[quickcheck]
    fn parses_arbitrary_repeat_loop(repeat_loop: RepeatLoop) {
        let tokens: Vec<_> = repeat_loop.clone().to_tokens().collect();
        let parsed = lua_parser::repeat_loop(&tokens).unwrap();
        assert_eq!(repeat_loop, parsed);
    }
}
