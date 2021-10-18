use std::iter;

use super::Expression;
use crate::{
    lex::{DynTokens, Ident, ToTokenStream, Token},
    util::flat_intersperse,
};

#[derive(Debug, PartialEq, Clone)]
pub enum TableConstructor {
    Empty,
    LFieldList(Vec<Expression>),
    FFieldList(Vec<(Ident, Expression)>),
    Combined {
        lfield: Vec<Expression>,
        ffield: Vec<(Ident, Expression)>,
    },
}

fn lfieldlist_tokens(lfield: Vec<Expression>) -> impl Iterator<Item = Token> {
    flat_intersperse(
        lfield.into_iter().map(ToTokenStream::to_tokens),
        Token::Comma,
    )
}

fn ffieldlist_tokens(ffield: Vec<(Ident, Expression)>) -> impl Iterator<Item = Token> {
    flat_intersperse(
        ffield.into_iter().map(|(name, expr)| {
            name.to_tokens()
                .chain(iter::once(Token::Assignment))
                .chain(expr.to_tokens())
        }),
        Token::Comma,
    )
}

impl ToTokenStream for TableConstructor {
    type Tokens = DynTokens;
    fn to_tokens(self) -> Self::Tokens {
        match self {
            TableConstructor::Empty => Box::new(IntoIterator::into_iter([
                Token::OpenSquigglyBracket,
                Token::CloseSquigglyBracket,
            ])),
            TableConstructor::LFieldList(exprs) => Box::new(
                iter::once(Token::OpenSquigglyBracket)
                    .chain(lfieldlist_tokens(exprs))
                    .chain(iter::once(Token::CloseSquigglyBracket)),
            ),
            TableConstructor::FFieldList(exprs) => Box::new(
                iter::once(Token::OpenSquigglyBracket)
                    .chain(ffieldlist_tokens(exprs))
                    .chain(iter::once(Token::CloseSquigglyBracket)),
            ),
            TableConstructor::Combined { lfield, ffield } => Box::new(
                iter::once(Token::OpenSquigglyBracket)
                    .chain(lfieldlist_tokens(lfield))
                    .chain(iter::once(Token::Comma))
                    .chain(ffieldlist_tokens(ffield))
                    .chain(iter::once(Token::CloseSquigglyBracket)),
                // .chain()
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use super::lfieldlist_tokens;
    use super::TableConstructor;
    use crate::lex::ToTokenStream;
    use crate::lex::Token;
    use crate::syn::lua_parser;
    use crate::syn::Expression;
    use crate::test_util::{arbitrary_recursive_vec, QUICKCHECK_RECURSIVE_DEPTH};
    use quickcheck::empty_shrinker;
    use quickcheck::{Arbitrary, Gen, TestResult};
    use std::iter;

    impl Arbitrary for TableConstructor {
        fn arbitrary(g: &mut Gen) -> Self {
            if g.size() == 0 {
                TableConstructor::Empty
            } else {
                let gen = &mut Gen::new(QUICKCHECK_RECURSIVE_DEPTH.min(g.size() - 1));
                TableConstructor::LFieldList(arbitrary_recursive_vec(gen))
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match self {
                TableConstructor::Empty => empty_shrinker(),
                TableConstructor::LFieldList(_) | TableConstructor::FFieldList(_) => {
                    Box::new(iter::once(TableConstructor::Empty))
                }
                TableConstructor::Combined { lfield, ffield } => {
                    Box::new(IntoIterator::into_iter([
                        TableConstructor::LFieldList(lfield.clone()),
                        TableConstructor::FFieldList(ffield.clone()),
                    ]))
                }
            }
        }
    }

    #[test]
    fn parses_empty_table_constructor() {
        let tokens = [Token::OpenSquigglyBracket, Token::CloseSquigglyBracket];
        let parsed = lua_parser::table_constructor(&tokens).unwrap();
        assert_eq!(parsed, TableConstructor::Empty);
    }

    #[quickcheck]
    #[ignore]
    fn parses_arbitrary_table_constructor(expected: TableConstructor) {
        let tokens = expected.clone().to_tokens().collect::<Vec<_>>();
        let parsed = lua_parser::table_constructor(&tokens).unwrap();
        assert_eq!(parsed, expected);
    }

    #[quickcheck]
    #[ignore]
    fn parses_arbitrary_list_table_constructor_with_trailing_comma(
        exprs: Vec<Expression>,
    ) -> TestResult {
        if exprs.len() == 0 {
            return TestResult::discard();
        }
        let mut tokens = Vec::new();
        tokens.push(Token::OpenSquigglyBracket);
        tokens.extend(lfieldlist_tokens(exprs.clone()));
        tokens.push(Token::Comma);
        tokens.push(Token::CloseSquigglyBracket);
        let parsed = lua_parser::table_constructor(&tokens).unwrap();
        let table_constructor = TableConstructor::LFieldList(exprs);
        assert_eq!(parsed, table_constructor);
        TestResult::passed()
    }

    #[quickcheck]
    #[ignore]
    fn parses_arbitrary_list_table_constructor(exprs: Vec<Expression>) -> TestResult {
        if exprs.len() == 0 {
            return TestResult::discard();
        }
        let mut tokens = Vec::new();
        tokens.push(Token::OpenSquigglyBracket);
        tokens.extend(lfieldlist_tokens(exprs.clone()));
        tokens.push(Token::CloseSquigglyBracket);
        let parsed = lua_parser::table_constructor(&tokens).unwrap();
        let table_constructor = TableConstructor::LFieldList(exprs);
        assert_eq!(parsed, table_constructor);
        TestResult::passed()
    }
}
