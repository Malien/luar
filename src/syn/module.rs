use crate::{
    fmt_tokens,
    lex::{DynTokens, ToTokenStream},
};

use super::{FunctionDeclaration, Return, Statement};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Module {
    pub chunks: Vec<Chunk>,
    pub ret: Option<Return>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Chunk {
    FnDecl(FunctionDeclaration),
    Statement(Statement),
}

impl ToTokenStream for Chunk {
    type Tokens = DynTokens;

    fn to_tokens(self) -> Self::Tokens {
        match self {
            Self::FnDecl(decl) => decl.to_tokens(),
            Self::Statement(statement) => statement.to_tokens(),
        }
    }
}

impl ToTokenStream for Module {
    type Tokens = DynTokens;

    fn to_tokens(self) -> Self::Tokens {
        Box::new(
            self.chunks
                .into_iter()
                .flat_map(ToTokenStream::to_tokens)
                .chain(self.ret.into_iter().flat_map(ToTokenStream::to_tokens)),
        )
    }
}

fmt_tokens!(Module);

#[cfg(test)]
mod test {
    use quickcheck::Arbitrary;

    use crate::{
        assert_parses,
        lex::Token,
        syn::{lua_parser, FunctionDeclaration, Return, Statement},
    };

    use super::{Chunk, Module};

    impl Arbitrary for Module {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Self {
                chunks: Arbitrary::arbitrary(g),
                ret: Arbitrary::arbitrary(g),
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(
                self.chunks
                    .shrink()
                    .map({
                        let ret = self.ret.clone();
                        move |chunks| Self {
                            chunks,
                            ret: ret.clone(),
                        }
                    })
                    .chain(self.ret.shrink().map({
                        let chunks = self.chunks.clone();
                        move |ret| Self {
                            chunks: chunks.clone(),
                            ret,
                        }
                    })),
            )
        }
    }

    impl Arbitrary for Chunk {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            if bool::arbitrary(g) {
                Chunk::Statement(Statement::arbitrary(g))
            } else {
                Chunk::FnDecl(FunctionDeclaration::arbitrary(g))
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match self {
                Chunk::Statement(stmnt) => Box::new(stmnt.shrink().map(Chunk::Statement)),
                Chunk::FnDecl(decl) => Box::new(decl.shrink().map(Chunk::FnDecl)),
            }
        }
    }

    fn parses(expected: Module) {
        assert_parses!(module, expected)
    }

    #[test]
    fn parses_empty_module() {
        assert_eq!(lua_parser::module(&[]).unwrap(), Module::default());
    }

    #[quickcheck]
    fn parses_arbitrary_statement_sequence(statements: Vec<Statement>) {
        parses(Module {
            chunks: statements.into_iter().map(Chunk::Statement).collect(),
            ret: None,
        });
    }

    #[quickcheck]
    fn parses_arbitrary_function_declaration_sequence(decls: Vec<FunctionDeclaration>) {
        parses(Module {
            chunks: decls.into_iter().map(Chunk::FnDecl).collect(),
            ret: None,
        })
    }

    #[quickcheck]
    fn parses_arbitrary_chunk_sequence(chunks: Vec<Chunk>) {
        parses(Module { chunks, ret: None })
    }

    #[test]
    fn parses_just_return() {
        assert_eq!(
            lua_parser::module(&[Token::Return]).unwrap(),
            Module {
                chunks: vec![],
                ret: Some(Return(None))
            }
        );
    }

    #[quickcheck]
    fn parses_arbitrary_module(module: Module) {
        parses(module);
    }
}
