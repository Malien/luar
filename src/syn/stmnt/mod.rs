use crate::{
    fmt_tokens,
    lex::{DynTokens, ToTokenStream},
};

mod assignment;
pub use assignment::*;

mod declaration;
pub use declaration::*;

mod while_loop;
pub use while_loop::*;

mod repeat_loop;
pub use repeat_loop::*;

mod conditional;
pub use conditional::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    LocalDeclaration(Declaration),
    While(WhileLoop),
    Repeat(RepeatLoop),
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
            Self::While(while_loop) => while_loop.to_tokens(),
            Self::Repeat(repeat_loop) => repeat_loop.to_tokens(),
        }
    }
}

fmt_tokens!(Statement);

#[cfg(test)]
mod test {
    use quickcheck::Arbitrary;

    use crate::lex::{ToTokenStream, Token};
    use crate::syn::lua_parser;

    use super::{Assignment, Declaration, RepeatLoop, Statement, WhileLoop};

    impl Arbitrary for Statement {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            // let g = &mut g.next_iter();
            match u8::arbitrary(g) % 4 {
                0 => Statement::Assignment(Assignment::arbitrary(g)),
                1 => Statement::LocalDeclaration(Declaration::arbitrary(g)),
                2 => Statement::While(WhileLoop::arbitrary(g)),
                3 => Statement::Repeat(RepeatLoop::arbitrary(g)),
                _ => unreachable!(),
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match self {
                Self::Assignment(a) => Box::new(a.shrink().map(Self::Assignment)),
                Self::LocalDeclaration(decl) => Box::new(decl.shrink().map(Self::LocalDeclaration)),
                Self::While(while_loop) => Box::new(while_loop.shrink().map(Self::While)),
                Self::Repeat(repeat_loop) => Box::new(repeat_loop.shrink().map(Self::Repeat)),
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
