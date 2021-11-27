use crate::{
    fmt_tokens,
    lex::{DynTokens, ToTokenStream},
};

mod assignment;
pub use assignment::*;

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

#[cfg(test)]
mod test {
    use quickcheck::Arbitrary;

    use crate::lex::{ToTokenStream, Token};
    use crate::syn::lua_parser;

    use super::Statement;

    impl Arbitrary for Statement {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Self::Assignment(Arbitrary::arbitrary(g))
        }
        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            let Self::Assignment(a) = self;
            Box::new(a.shrink().map(Self::Assignment))
        }
    }

    #[quickcheck]
    fn parses_arbitrary_statement(expected: Statement) {
        let tokens = expected.clone().to_tokens().collect::<Vec<_>>();
        let parsed = lua_parser::statement(&tokens).unwrap();
        assert_eq!(parsed, expected);
    }

    #[quickcheck]
    fn parses_arbitrary_statement_ending_with_semi(expected: Statement) {
        let tokens: Vec<_> = expected
            .clone()
            .to_tokens()
            .chain(std::iter::once(Token::Semicolon))
            .collect();
        let parsed = lua_parser::statement(&tokens).unwrap();
        assert_eq!(parsed, expected);
    }
}
