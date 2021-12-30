use std::iter::*;

use crate::{
    fmt_tokens,
    lex::{DynTokens, ToTokenStream, Token},
    syn::expr::Expression,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Return(pub Option<Expression>);

impl ToTokenStream for Return {
    type Tokens = Chain<Once<Token>, Flatten<std::option::IntoIter<DynTokens>>>;

    fn to_tokens(self) -> Self::Tokens {
        std::iter::once(Token::Return)
            .chain(self.0.map(ToTokenStream::to_tokens).into_iter().flatten())
    }
}

fmt_tokens!(Return);

#[cfg(test)]
mod test {
    use quickcheck::{empty_shrinker, Arbitrary, Gen};

    use crate::{
        lex::{ToTokenStream, Token},
        syn::{expr::Expression, lua_parser, Return},
    };

    impl Arbitrary for Return {
        fn arbitrary(g: &mut Gen) -> Self {
            Self(Arbitrary::arbitrary(g))
        }
        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match self {
                Return(None) => empty_shrinker(),
                Return(Some(expression)) => Box::new(
                    std::iter::once(Return(None)).chain(expression.shrink().map(Some).map(Return)),
                ),
            }
        }
    }

    #[test]
    fn correctly_displays_empty_return() {
        assert_eq!(format!("{}", Return(None)), "return");
    }

    #[quickcheck]
    fn correctly_displays_return(expression: Expression) {
        assert_eq!(
            format!("{}", Return(Some(expression.clone()))),
            format!("return {}", expression)
        );
    }

    #[test]
    fn parses_empty_return() {
        let parsed = lua_parser::ret(&[Token::Return]).unwrap();
        assert_eq!(parsed, Return(None));
    }

    #[quickcheck]
    fn parses_arbitrary_expression_return(expression: Expression) {
        let expected = Return(Some(expression));
        let tokens: Vec<_> = expected.clone().to_tokens().collect();
        let parsed = lua_parser::ret(&tokens).unwrap();
        assert_eq!(parsed, expected);
    }
}
