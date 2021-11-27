use crate::{
    fmt_tokens,
    lex::{DynTokens, ToTokenStream},
};

mod assignment;
pub use assignment::*;

mod declaration;
pub use declaration::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    LocalDeclaration(Declaration),
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
            Self::Assignment(assignment) => assignment.to_tokens(),
            Self::LocalDeclaration(decl) => decl.to_tokens(),
        }
    }
}

fmt_tokens!(Statement);

#[cfg(test)]
mod test {
    use quickcheck::Arbitrary;

    use crate::lex::{ToTokenStream, Token};
    use crate::syn::lua_parser;
    use crate::test_util::GenExt;

    use super::{Assignment, Declaration, Statement};

    impl Arbitrary for Statement {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let g = &mut g.next_iter();
            match u8::arbitrary(g) % 2 {
                0 => Statement::Assignment(Assignment::arbitrary(g)),
                1 => Statement::LocalDeclaration(Declaration::arbitrary(g)),
                _ => unreachable!()
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match self {
                Self::Assignment(a) => Box::new(a.shrink().map(Self::Assignment)),
                Self::LocalDeclaration(decl) => Box::new(decl.shrink().map(Self::LocalDeclaration)),
            }
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
