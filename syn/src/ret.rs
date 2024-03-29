use std::iter::*;

use luar_lex::{fmt_tokens, DynTokens, ToTokenStream, Token};

use crate::{expr::Expression, flat_intersperse::FlatIntersperseExt};

#[derive(Debug, Clone, PartialEq)]
pub struct Return(pub Vec<Expression>);

impl ToTokenStream for Return {
    type Tokens = DynTokens;

    fn to_tokens(self) -> Self::Tokens {
        // I hate this!
        Box::new(
            std::iter::once(Token::Return).chain(
                self.0
                    .into_iter()
                    .map(ToTokenStream::to_tokens)
                    .flat_intersperse(Token::Comma),
            ),
        )
    }
}

impl Return {
    pub fn single(expression: Expression) -> Self {
        Self(vec![expression])
    }
    pub fn empty() -> Self {
        Self(vec![])
    }
}

fmt_tokens!(Return);

#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for Return {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self(quickcheck::Arbitrary::arbitrary(g))
    }
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(Return))
    }
}

#[cfg(test)]
mod test {
    use luar_lex::Token;

    use crate::{unspanned_lua_token_parser, Return};

    #[cfg(feature = "quickcheck")]
    use crate::{flat_intersperse::FlatIntersperseExt, Expression};
    #[cfg(feature = "quickcheck")]
    use ::{
        luar_lex::{format::format_tokens, ToTokenStream},
        non_empty::NonEmptyVec,
    };

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn correctly_displays_empty_return() {
        assert_eq!(format!("{}", Return::empty()), "return");
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn correctly_displays_return(expression: Expression) {
        assert_eq!(
            format!("{}", Return::single(expression.clone())),
            format!("return {}", expression)
        );
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn correctly_displays_multiple_return(expressions: NonEmptyVec<Expression>) {
        let mut buf = String::new();
        let mut tokens = expressions
            .clone()
            .into_iter()
            .map(ToTokenStream::to_tokens)
            .flat_intersperse(Token::Comma);
        format_tokens(&mut tokens, &mut buf).unwrap();
        assert_eq!(
            format!("{}", Return(expressions.into())),
            format!("return {}", buf)
        );
    }

    #[test]
    fn parses_empty_return() {
        let tokens = [Token::Return];
        let parsed = unspanned_lua_token_parser::ret(tokens).unwrap();
        assert_eq!(parsed, Return::empty());
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn parses_arbitrary_expression_return(expression: Expression) {
        let expected = Return::single(expression);
        let tokens: Vec<_> = expected.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::ret(tokens).unwrap();
        assert_eq!(parsed, expected);
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn parses_arbitrary_multiple_return(expressions: Vec<Expression>) {
        let expected = Return(expressions.clone());
        let tokens: Vec<_> = std::iter::once(Token::Return)
            .chain(
                expressions
                    .into_iter()
                    .map(ToTokenStream::to_tokens)
                    .flat_intersperse(Token::Comma),
            )
            .collect();
        let parsed = unspanned_lua_token_parser::ret(tokens).unwrap();
        assert_eq!(parsed, expected);
    }
}
