use std::iter::*;

use crate::{
    fmt_tokens,
    lex::{DynTokens, ToTokenStream, Token},
    syn::expr::Expression,
    util::{FlatIntersperseExt, NonEmptyVec},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Return(pub NonEmptyVec<Expression>);

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
        Self(NonEmptyVec::of_single(expression))
    }
}

fmt_tokens!(Return);

#[cfg(test)]
mod test {
    use quickcheck::{Arbitrary, Gen};

    use crate::{
        lex::{format::format_tokens, ToTokenStream, Token},
        syn::{expr::Expression, lua_parser, Return},
        util::{FlatIntersperseExt, NonEmptyVec},
    };

    impl Arbitrary for Return {
        fn arbitrary(g: &mut Gen) -> Self {
            Self(Arbitrary::arbitrary(g))
        }
        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(self.0.shrink().map(Return))
        }
    }

    #[quickcheck]
    fn correctly_displays_return(expression: Expression) {
        assert_eq!(
            format!("{}", Return::single(expression.clone())),
            format!("return {}", expression)
        );
    }

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
            format!("{}", Return(expressions)),
            format!("return {}", buf)
        );
    }

    #[quickcheck]
    fn parses_arbitrary_expression_return(expression: Expression) {
        let expected = Return::single(expression);
        let tokens: Vec<_> = expected.clone().to_tokens().collect();
        let parsed = lua_parser::ret(&tokens).unwrap();
        assert_eq!(parsed, expected);
    }

    #[quickcheck]
    fn parses_arbitrary_multiple_return(expressions: NonEmptyVec<Expression>) {
        let expected = Return(expressions.clone());
        let tokens: Vec<_> = std::iter::once(Token::Return)
            .chain(
                expressions
                    .into_iter()
                    .map(ToTokenStream::to_tokens)
                    .flat_intersperse(Token::Comma),
            )
            .collect();
        let parsed = lua_parser::ret(&tokens).unwrap();
        assert_eq!(parsed, expected);
    }
}
