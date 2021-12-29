use crate::{
    fmt_tokens,
    lex::{DynTokens, ToTokenStream, Token},
    syn::expr::Expression,
};

use super::Statement;

#[derive(Debug, Clone, PartialEq)]
pub struct Conditional {
    pub condition: Expression,
    pub body: Vec<Statement>,
    pub tail: ConditionalTail,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionalTail {
    End,
    Else(Vec<Statement>),
    ElseIf(Box<Conditional>),
}

impl ToTokenStream for Conditional {
    type Tokens = DynTokens;

    fn to_tokens(self) -> Self::Tokens {
        Box::new(
            std::iter::once(Token::If)
                .chain(self.condition.to_tokens())
                .chain(std::iter::once(Token::Then))
                .chain(self.body.into_iter().flat_map(ToTokenStream::to_tokens))
                .chain(self.tail.to_tokens()),
        )
    }
}

impl ToTokenStream for ConditionalTail {
    type Tokens = DynTokens;
    fn to_tokens(self) -> Self::Tokens {
        match self {
            ConditionalTail::End => Box::new(std::iter::once(Token::End)),
            ConditionalTail::Else(body) => Box::new(
                std::iter::once(Token::Else)
                    .chain(body.into_iter().flat_map(ToTokenStream::to_tokens))
                    .chain(std::iter::once(Token::End)),
            ),
            ConditionalTail::ElseIf(conditional) => {
                let Conditional {
                    condition,
                    body,
                    tail,
                } = *conditional;

                Box::new(
                    std::iter::once(Token::ElseIf)
                        .chain(condition.to_tokens())
                        .chain(std::iter::once(Token::Then))
                        .chain(body.into_iter().flat_map(ToTokenStream::to_tokens))
                        .chain(tail.to_tokens()),
                )
            }
        }
    }
}

fmt_tokens!(Conditional);

#[cfg(test)]
mod test {
    use quickcheck::{Arbitrary, Gen};

    use crate::{
        input_parsing_expectation,
        lex::{Ident, NumberLiteral, ToTokenStream},
        syn::{expr::Expression, Declaration, Statement, lua_parser},
        test_util::{arbitrary_recursive_vec, GenExt},
        util::NonEmptyVec,
    };

    use super::{Conditional, ConditionalTail};

    impl Arbitrary for Conditional {
        fn arbitrary(g: &mut Gen) -> Self {
            if g.size() <= 1 {
                return Self {
                    condition: Expression::Nil,
                    body: vec![],
                    tail: ConditionalTail::End,
                };
            }
            let mut inner_gen = g.next_iter();
            Self {
                condition: Expression::arbitrary(&mut inner_gen),
                body: arbitrary_recursive_vec(&mut inner_gen),
                tail: ConditionalTail::arbitrary(&mut inner_gen),
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(
                self.condition
                    .shrink()
                    .map({
                        let body = self.body.clone();
                        let tail = self.tail.clone();
                        move |condition| Self {
                            condition,
                            body: body.clone(),
                            tail: tail.clone(),
                        }
                    })
                    .chain(self.body.shrink().map({
                        let tail = self.tail.clone();
                        let condition = self.condition.clone();
                        move |body| Self {
                            condition: condition.clone(),
                            body,
                            tail: tail.clone(),
                        }
                    }))
                    .chain(self.tail.shrink().map({
                        let body = self.body.clone();
                        let condition = self.condition.clone();
                        move |tail| Self {
                            condition: condition.clone(),
                            body: body.clone(),
                            tail,
                        }
                    })),
            )
        }
    }

    impl Arbitrary for ConditionalTail {
        fn arbitrary(g: &mut Gen) -> Self {
            if g.size() <= 1 {
                return ConditionalTail::End;
            }
            match u8::arbitrary(g) % 3 {
                0 => ConditionalTail::End,
                1 => ConditionalTail::Else(Arbitrary::arbitrary(g)),
                2 => ConditionalTail::ElseIf(Arbitrary::arbitrary(g)),
                _ => unreachable!(),
            }
        }
    }

    macro_rules! test_display {
        ($name: tt, $input: expr, $output: expr) => {
            #[test]
            fn $name() {
                assert_eq!(format!("{}", $input), $output);
            }
        };
    }

    test_display!(
        simple_clause,
        Conditional {
            condition: Expression::Nil,
            body: vec![Statement::LocalDeclaration(Declaration {
                names: NonEmptyVec::of_single(Ident::new("foo")),
                initial_values: vec![Expression::Nil],
            })],
            tail: ConditionalTail::End,
        },
        "if nil then\n\tlocal foo = nil\nend"
    );

    test_display!(
        else_clause,
        Conditional {
            condition: Expression::Nil,
            body: vec![Statement::LocalDeclaration(Declaration {
                names: NonEmptyVec::of_single(Ident::new("foo")),
                initial_values: vec![Expression::Nil],
            })],
            tail: ConditionalTail::Else(vec![Statement::LocalDeclaration(Declaration {
                names: NonEmptyVec::of_single(Ident::new("bar")),
                initial_values: vec![Expression::Nil],
            })])
        },
        "if nil then\n\tlocal foo = nil\nelse\n\tlocal bar = nil\nend"
    );

    test_display!(
        elseif_end_clause,
        Conditional {
            condition: Expression::Nil,
            body: vec![Statement::LocalDeclaration(Declaration {
                names: NonEmptyVec::of_single(Ident::new("foo")),
                initial_values: vec![Expression::Nil],
            })],
            tail: ConditionalTail::ElseIf(Box::new(Conditional {
                condition: Expression::Nil,
                body: vec![Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(Ident::new("bar")),
                    initial_values: vec![Expression::Nil],
                })],
                tail: ConditionalTail::End
            }))
        },
        "if nil then\n\tlocal foo = nil\nelseif nil then\n\tlocal bar = nil\nend"
    );

    test_display!(
        elseif_else_clause,
        Conditional {
            condition: Expression::Nil,
            body: vec![Statement::LocalDeclaration(Declaration {
                names: NonEmptyVec::of_single(Ident::new("foo")),
                initial_values: vec![Expression::Nil],
            })],
            tail: ConditionalTail::ElseIf(Box::new(Conditional {
                condition: Expression::Nil,
                body: vec![Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(Ident::new("bar")),
                    initial_values: vec![Expression::Nil],
                })],
                tail: ConditionalTail::Else(vec![Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(Ident::new("baz")),
                    initial_values: vec![Expression::Nil],
                })])
            }))
        },
        "if nil then\n\tlocal foo = nil\nelseif nil then\n\tlocal bar = nil\nelse\n\tlocal baz = nil\nend"
    );

    test_display!(
        elseif_elseif_clause,
        Conditional {
            condition: Expression::Nil,
            body: vec![Statement::LocalDeclaration(Declaration {
                names: NonEmptyVec::of_single(Ident::new("foo")),
                initial_values: vec![Expression::Nil],
            })],
            tail: ConditionalTail::ElseIf(Box::new(Conditional {
                condition: Expression::Nil,
                body: vec![Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(Ident::new("bar")),
                    initial_values: vec![Expression::Nil],
                })],
                tail: ConditionalTail::ElseIf(Box::new(Conditional {
                    condition: Expression::Nil,
                    body: vec![Statement::LocalDeclaration(Declaration {
                        names: NonEmptyVec::of_single(Ident::new("baz")),
                        initial_values: vec![Expression::Nil],
                    })],
                    tail: ConditionalTail::End
                }))
            }))
        },
        "if nil then\n\tlocal foo = nil\nelseif nil then\n\tlocal bar = nil\nelseif nil then\n\tlocal baz = nil\nend"
    );

    input_parsing_expectation!(
        conditional,
        parses_empty_if_clause,
        "if nil then end",
        Conditional {
            body: vec![],
            condition: Expression::Nil,
            tail: ConditionalTail::End
        }
    );

    input_parsing_expectation!(
        conditional,
        parses_simple_if_clause,
        "if nil then
            local foo = 42
            local bar = 69
        end",
        Conditional {
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
            condition: Expression::Nil,
            tail: ConditionalTail::End
        }
    );

    input_parsing_expectation!(
        conditional,
        parses_else_clause,
        "if nil then
            local foo = 42
        else
            local bar = 69
        end",
        Conditional {
            body: vec![Statement::LocalDeclaration(Declaration {
                names: NonEmptyVec::of_single(Ident::new("foo")),
                initial_values: vec![Expression::Number(NumberLiteral(42f64))]
            })],
            condition: Expression::Nil,
            tail: ConditionalTail::Else(vec![Statement::LocalDeclaration(Declaration {
                names: NonEmptyVec::of_single(Ident::new("bar")),
                initial_values: vec![Expression::Number(NumberLiteral(69f64))]
            })]),
        }
    );

    input_parsing_expectation!(
        conditional,
        parses_elseif_clause,
        "if nil then
            local foo = 42
        elseif nil then
            local bar = 69
        end",
        Conditional {
            body: vec![Statement::LocalDeclaration(Declaration {
                names: NonEmptyVec::of_single(Ident::new("foo")),
                initial_values: vec![Expression::Number(NumberLiteral(42f64))]
            })],
            condition: Expression::Nil,
            tail: ConditionalTail::ElseIf(Box::new(Conditional {
                body: vec![Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(Ident::new("bar")),
                    initial_values: vec![Expression::Number(NumberLiteral(69f64))]
                })],
                condition: Expression::Nil,
                tail: ConditionalTail::End
            })),
        }
    );

    input_parsing_expectation!(
        conditional,
        parses_elseif_else_clause,
        "if nil then
            local foo = 42
        elseif nil then
            local bar = 69
        else 
            local baz = nil
        end",
        Conditional {
            body: vec![Statement::LocalDeclaration(Declaration {
                names: NonEmptyVec::of_single(Ident::new("foo")),
                initial_values: vec![Expression::Number(NumberLiteral(42f64))]
            })],
            condition: Expression::Nil,
            tail: ConditionalTail::ElseIf(Box::new(Conditional {
                body: vec![Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(Ident::new("bar")),
                    initial_values: vec![Expression::Number(NumberLiteral(69f64))]
                })],
                condition: Expression::Nil,
                tail: ConditionalTail::Else(vec![Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(Ident::new("baz")),
                    initial_values: vec![Expression::Nil]
                })])
            })),
        }
    );

    #[quickcheck]
    fn parses_arbitrary_conditional(conditional: Conditional) {
        let tokens: Vec<_> = conditional.clone().to_tokens().collect();
        let parsed = lua_parser::conditional(&tokens).unwrap();
        assert_eq!(parsed, conditional);
    }
}
