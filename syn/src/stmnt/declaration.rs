use luar_lex::{fmt_tokens, DynTokens, Ident, ToTokenStream, Token};
use non_empty::NonEmptyVec;

use crate::{expr::Expression, flat_intersperse::FlatIntersperseExt};

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub names: NonEmptyVec<Ident>,
    pub initial_values: Vec<Expression>,
}

impl ToTokenStream for Declaration {
    type Tokens = DynTokens;
    fn to_tokens(self) -> Self::Tokens {
        let decl = std::iter::once(Token::Local).chain(
            self.names
                .into_iter()
                .map(ToTokenStream::to_tokens)
                .flat_intersperse(Token::Comma),
        );
        if self.initial_values.is_empty() {
            Box::new(decl)
        } else {
            Box::new(
                decl.chain(std::iter::once(Token::Assignment)).chain(
                    self.initial_values
                        .into_iter()
                        .map(ToTokenStream::to_tokens)
                        .flat_intersperse(Token::Comma),
                ),
            )
        }
    }
}

fmt_tokens!(Declaration);

#[cfg(feature = "quickcheck")]
use quickcheck::{Arbitrary, Gen};

#[cfg(feature = "quickcheck")]
impl Arbitrary for Declaration {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            names: Arbitrary::arbitrary(g),
            initial_values: Arbitrary::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let names = self.names.clone();
        let initial_values = self.initial_values.clone();
        Box::new(
            self.names
                .shrink()
                .map(move |names| Self {
                    names,
                    initial_values: initial_values.clone(),
                })
                .chain(
                    self.initial_values
                        .shrink()
                        .map(move |initial_values| Self {
                            names: names.clone(),
                            initial_values,
                        }),
                ),
        )
    }
}

#[cfg(test)]
mod test {
    use logos::Logos;
    use luar_lex::{Ident, NumberLiteral, Token};
    use non_empty::NonEmptyVec;

    use crate::{expr::Expression, input_parsing_expectation, unspanned_lua_token_parser};

    use super::Declaration;

    #[cfg(feature = "quickcheck")]
    use luar_lex::ToTokenStream;

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn parses_arbitrary_declaration(decl: Declaration) {
        let tokens: Vec<_> = decl.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::declaration(tokens).unwrap();
        assert_eq!(decl, parsed);
    }

    input_parsing_expectation!(
        declaration,
        single_declaration,
        "local a",
        Declaration {
            names: NonEmptyVec::of_single(Ident::new("a")),
            initial_values: vec![]
        }
    );

    input_parsing_expectation!(
        declaration,
        multiple_declarations,
        "local a, b",
        Declaration {
            names: unsafe { NonEmptyVec::new_unchecked(vec![Ident::new("a"), Ident::new("b"),]) },
            initial_values: vec![]
        }
    );

    #[test]
    fn zero_declarations_is_illegal() {
        let tokens = [Token::Local];
        let res = unspanned_lua_token_parser::declaration(tokens);
        assert!(res.is_err());
    }

    input_parsing_expectation!(
        declaration,
        single_initialization,
        "local a = 42",
        Declaration {
            names: NonEmptyVec::of_single(Ident::new("a")),
            initial_values: vec![Expression::Number(NumberLiteral(42f64))]
        }
    );

    input_parsing_expectation!(
        declaration,
        multiple_initialization,
        "local a, b = 42, 69",
        Declaration {
            names: unsafe { NonEmptyVec::new_unchecked(vec![Ident::new("a"), Ident::new("b"),]) },
            initial_values: vec![
                Expression::Number(NumberLiteral(42f64)),
                Expression::Number(NumberLiteral(69f64))
            ]
        }
    );

    #[test]
    fn initialization_without_declaration_is_illegal() {
        let tokens: Vec<_> = Token::lexer("local = 42").collect();
        let res = unspanned_lua_token_parser::declaration(tokens);
        assert!(res.is_err());
    }
}
