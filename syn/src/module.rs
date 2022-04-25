use luar_lex::{fmt_tokens, DynTokens, ToTokenStream};

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

impl Chunk {
    pub fn as_statement(self) -> Option<Statement> {
        match self {
            Chunk::FnDecl(_) => None,
            Chunk::Statement(statement) => Some(statement)
        }
    }
    pub fn as_statement_ref(&self) -> Option<&Statement> {
        match self {
            Chunk::FnDecl(_) => None,
            Chunk::Statement(statement) => Some(statement)
        }
    }
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

#[cfg(feature = "quickcheck")]
use quickcheck::{Arbitrary, Gen};

#[cfg(feature = "quickcheck")]
impl Arbitrary for Module {
    fn arbitrary(g: &mut Gen) -> Self {
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

#[cfg(feature = "quickcheck")]
impl Arbitrary for Chunk {
    fn arbitrary(g: &mut Gen) -> Self {
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

#[cfg(test)]
mod test {
    use crate::lua_parser;

    use super::Module;

    #[cfg(feature = "quickcheck")]
    fn parses(expected: Module) {
        assert_parses!(module, expected)
    }

    #[cfg(feature = "quickcheck")]
    use crate::{Chunk, FunctionDeclaration, Return, Statement, assert_parses};

    #[test]
    fn parses_empty_module() {
        assert_eq!(lua_parser::module("").unwrap(), Module::default());
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn parses_arbitrary_statement_sequence(statements: Vec<Statement>) {
        parses(Module {
            chunks: statements.into_iter().map(Chunk::Statement).collect(),
            ret: None,
        });
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn parses_arbitrary_function_declaration_sequence(decls: Vec<FunctionDeclaration>) {
        parses(Module {
            chunks: decls.into_iter().map(Chunk::FnDecl).collect(),
            ret: None,
        })
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn parses_arbitrary_chunk_sequence(chunks: Vec<Chunk>) {
        parses(Module { chunks, ret: None })
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn parses_just_arbitrary_return(ret: Return) {
        parses(Module {
            chunks: vec![],
            ret: Some(ret),
        })
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn parses_arbitrary_module(module: Module) {
        parses(module);
    }
}
