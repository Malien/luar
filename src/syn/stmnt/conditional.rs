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
    use crate::{
        lex::Ident,
        syn::{expr::Expression, Declaration, Statement},
        util::NonEmptyVec,
    };

    use super::{Conditional, ConditionalTail};

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
}
