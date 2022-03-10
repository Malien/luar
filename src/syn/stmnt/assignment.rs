use crate::{
    fmt_tokens,
    lex::{DynTokens, ToTokenStream, Token},
    syn::expr::{Expression, Var},
    util::{FlatIntersperseExt, NonEmptyVec},
};

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
    use super::Assignment;
    use crate::{lex::ToTokenStream, syn::unspanned_lua_token_parser};
    use quickcheck::Arbitrary;

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
        let parsed = unspanned_lua_token_parser::assignment(tokens).unwrap();
        assert_eq!(parsed, expected);
    }
}
