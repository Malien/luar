use luar_lex::{fmt_tokens, DynTokens, ToTokenStream};

use super::{Return, Statement};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub ret: Option<Return>,
}

impl ToTokenStream for Block {
    // This type can be named, but god damn it is tedious!
    // Waiting for `type Tokens = impl Iterator<Item = Token>` to become a reality!
    type Tokens = DynTokens;

    fn to_tokens(self) -> Self::Tokens {
        Box::new(
            self.statements
                .into_iter()
                .flat_map(ToTokenStream::to_tokens)
                .chain(self.ret.map(ToTokenStream::to_tokens).into_iter().flatten()),
        )
    }
}

fmt_tokens!(Block);

#[cfg(test)]
mod test {
    use luar_lex::{format::format_tokens, ToTokenStream};
    use quickcheck::{Arbitrary, Gen};

    use crate::{
        assert_parses,
        syn::{Return, Statement},
        test_util::GenExt,
    };

    use super::Block;

    impl Arbitrary for Block {
        fn arbitrary(g: &mut Gen) -> Self {
            if g.size() <= 1 {
                return Block::default();
            }
            let g = &mut g.next_iter();
            Self {
                statements: Arbitrary::arbitrary(g),
                ret: Arbitrary::arbitrary(g),
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(
                self.statements
                    .shrink()
                    .map({
                        let ret = self.ret.clone();
                        move |statements| Block {
                            statements,
                            ret: ret.clone(),
                        }
                    })
                    .chain(self.ret.shrink().map({
                        let statements = self.statements.clone();
                        move |ret| Block {
                            statements: statements.clone(),
                            ret,
                        }
                    })),
            )
        }
    }

    #[quickcheck]
    fn displays_correctly_statements(statements: Vec<Statement>) {
        let expected = Block {
            statements: statements.clone(),
            ret: None,
        };
        let mut output = String::new();
        format_tokens(
            &mut statements.into_iter().flat_map(ToTokenStream::to_tokens),
            &mut output,
        )
        .unwrap();
        assert_eq!(format!("{}", expected), output);
    }

    #[quickcheck]
    fn displays_correctly_statements_with_arbitrary_return(
        statements: Vec<Statement>,
        ret: Return,
    ) {
        let expected = Block {
            statements: statements.clone(),
            ret: Some(ret.clone()),
        };
        let mut output = String::new();
        format_tokens(
            &mut statements
                .into_iter()
                .flat_map(ToTokenStream::to_tokens)
                .chain(ret.to_tokens()),
            &mut output,
        )
        .unwrap();
        assert_eq!(format!("{}", expected), output);
    }

    fn parse(expected: Block) {
        assert_parses!(block, expected)
    }

    #[quickcheck]
    fn parses_arbitrary_statements_block(statements: Vec<Statement>) {
        parse(Block {
            statements,
            ret: None,
        })
    }

    #[quickcheck]
    fn parses_arbitrary_return_block(ret: Return) {
        parse(Block {
            statements: vec![],
            ret: Some(ret),
        })
    }

    #[quickcheck]
    fn parses_arbitrary_block(expected: Block) {
        parse(expected)
    }
}
