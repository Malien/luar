use crate::{
    fmt_tokens,
    lex::{DynTokens, ToTokenStream, Token},
    util::{FlatIntersperseExt, NonEmptyVec},
};

use super::expr::{Expression, Var};

pub mod for_st;
pub mod if_st;
pub mod while_st;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    // LocalDeclaration,
    // While,
    // Repeat,
    // If,
    // ElseIf,
    // Return,
    // FunctionCall
}

impl ToTokenStream for Statement {
    type Tokens = DynTokens;
    fn to_tokens(self) -> Self::Tokens {
        match self {
            Self::Assignment(assignment) => Box::new(assignment.to_tokens()),
        }
    }
}

fmt_tokens!(Statement);

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub names: NonEmptyVec<Var>,
    pub values: NonEmptyVec<Expression>,
}

impl ToTokenStream for Assignment {
    type Tokens = DynTokens;
    fn to_tokens(self) -> Self::Tokens {
        let Self { names, values } = self;
        Box::new(
            names
                .into_iter()
                .map(ToTokenStream::to_tokens)
                .flat_intersperse(Token::Comma)
                .chain(std::iter::once(Token::Assignment))
                .chain(
                    values
                        .into_iter()
                        .map(ToTokenStream::to_tokens)
                        .flat_intersperse(Token::Comma),
                ),
        )
    }
}

fmt_tokens!(Assignment);

#[cfg(test)]
mod test {
    use quickcheck::Arbitrary;

    use crate::lex::ToTokenStream;
    use crate::syn::lua_parser;

    use super::Assignment;

    impl Arbitrary for Assignment {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Self {
                names: Arbitrary::arbitrary(g),
                values: Arbitrary::arbitrary(g),
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            let values = self.values.clone();
            let names = self.names.clone();
            Box::new(
                self.names
                    .shrink()
                    .map(move |names| Self {
                        names,
                        values: values.clone(),
                    })
                    .chain(self.values.shrink().map(move |values| Self {
                        values,
                        names: names.clone(),
                    })),
            )
        }
    }

    #[quickcheck]
    fn parses_arbitrary_assignment(expected: Assignment) {
        let tokens = expected.clone().to_tokens().collect::<Vec<_>>();
        let parsed = lua_parser::assignment(&tokens).unwrap();
        assert_eq!(parsed, expected);
    }
}
