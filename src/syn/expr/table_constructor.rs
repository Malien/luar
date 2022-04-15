use std::iter;

use luar_lex::{fmt_tokens, DynTokens, Ident, ToTokenStream, Token};

use super::Expression;
use crate::util::FlatIntersperseExt;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct TableConstructor {
    pub lfield: Vec<Expression>,
    pub ffield: Vec<(Ident, Expression)>,
}

#[allow(dead_code)]
impl TableConstructor {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn lfieldlist(lfield: Vec<Expression>) -> Self {
        Self {
            lfield,
            ..Default::default()
        }
    }

    pub fn ffieldlist(ffield: Vec<(Ident, Expression)>) -> Self {
        Self {
            ffield,
            ..Default::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lfield.is_empty() && self.ffield.is_empty()
    }
}

fn lfieldlist_tokens(lfield: Vec<Expression>) -> impl Iterator<Item = Token> {
    lfield
        .into_iter()
        .map(ToTokenStream::to_tokens)
        .flat_intersperse(Token::Comma)
}

fn ffieldlist_tokens(ffield: Vec<(Ident, Expression)>) -> impl Iterator<Item = Token> {
    ffield
        .into_iter()
        .map(|(name, expr)| {
            name.to_tokens()
                .chain(iter::once(Token::Assignment))
                .chain(expr.to_tokens())
        })
        .flat_intersperse(Token::Comma)
}

impl ToTokenStream for TableConstructor {
    type Tokens = DynTokens;
    fn to_tokens(self) -> Self::Tokens {
        match (self.lfield.is_empty(), self.ffield.is_empty()) {
            (true, true) => Box::new(IntoIterator::into_iter([
                Token::OpenSquigglyBracket,
                Token::CloseSquigglyBracket,
            ])),
            (false, true) => Box::new(
                iter::once(Token::OpenSquigglyBracket)
                    .chain(lfieldlist_tokens(self.lfield))
                    .chain(iter::once(Token::CloseSquigglyBracket)),
            ),
            (true, false) => Box::new(
                iter::once(Token::OpenSquigglyBracket)
                    .chain(ffieldlist_tokens(self.ffield))
                    .chain(iter::once(Token::CloseSquigglyBracket)),
            ),
            (false, false) => Box::new(
                iter::once(Token::OpenSquigglyBracket)
                    .chain(lfieldlist_tokens(self.lfield))
                    .chain(iter::once(Token::Semicolon))
                    .chain(ffieldlist_tokens(self.ffield))
                    .chain(iter::once(Token::CloseSquigglyBracket)),
            ),
        }
    }
}

fmt_tokens!(TableConstructor);

#[cfg(test)]
mod test {
    use super::lfieldlist_tokens;
    use super::TableConstructor;
    use crate::syn::unspanned_lua_token_parser;
    use crate::syn::Expression;
    use crate::test_util::arbitrary_recursive_vec;
    use crate::test_util::with_thread_gen;
    use crate::test_util::QUICKCHECK_RECURSIVE_DEPTH;
    use luar_lex::Ident;
    use luar_lex::ToTokenStream;
    use luar_lex::Token;
    use quickcheck::empty_shrinker;
    use quickcheck::{Arbitrary, Gen, TestResult};
    use std::iter;

    impl Arbitrary for TableConstructor {
        fn arbitrary(g: &mut Gen) -> Self {
            if g.size() == 0 {
                TableConstructor::empty()
            } else {
                let gen = &mut Gen::new(QUICKCHECK_RECURSIVE_DEPTH.min(g.size() - 1));
                let exprs = arbitrary_recursive_vec(gen);
                match u8::arbitrary(gen) % 3 {
                    0 => TableConstructor::lfieldlist(exprs),
                    1 => TableConstructor::ffieldlist(
                        exprs
                            .into_iter()
                            .map(|expr| (with_thread_gen(Ident::arbitrary), expr))
                            .collect(),
                    ),
                    2 => TableConstructor {
                        lfield: exprs,
                        ffield: arbitrary_recursive_vec(gen)
                            .into_iter()
                            .map(|expr| (with_thread_gen(Ident::arbitrary), expr))
                            .collect(),
                    },
                    _ => unreachable!(),
                }
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match (self.lfield.is_empty(), self.ffield.is_empty()) {
                (true, true) => empty_shrinker(),
                (true, false) | (false, true) => Box::new(iter::once(TableConstructor::empty())),
                (false, false) => Box::new(IntoIterator::into_iter([
                    TableConstructor::lfieldlist(self.lfield.clone()),
                    TableConstructor::ffieldlist(self.ffield.clone()),
                ])),
            }
        }
    }

    #[test]
    fn parses_empty_table_constructor() {
        let tokens = [Token::OpenSquigglyBracket, Token::CloseSquigglyBracket];
        let parsed = unspanned_lua_token_parser::table_constructor(tokens).unwrap();
        assert_eq!(parsed, TableConstructor::empty());
    }

    #[quickcheck]
    fn parses_arbitrary_table_constructor(expected: TableConstructor) {
        let tokens = expected.clone().to_tokens().collect::<Vec<_>>();
        let parsed = unspanned_lua_token_parser::table_constructor(tokens).unwrap();
        assert_eq!(parsed, expected);
    }

    #[quickcheck]
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
        let parsed = unspanned_lua_token_parser::table_constructor(tokens).unwrap();
        let table_constructor = TableConstructor::lfieldlist(exprs);
        assert_eq!(parsed, table_constructor);
        TestResult::passed()
    }

    #[quickcheck]
    fn parses_arbitrary_list_table_constructor(exprs: Vec<Expression>) -> TestResult {
        if exprs.len() == 0 {
            return TestResult::discard();
        }
        let mut tokens = Vec::new();
        tokens.push(Token::OpenSquigglyBracket);
        tokens.extend(lfieldlist_tokens(exprs.clone()));
        tokens.push(Token::CloseSquigglyBracket);
        let parsed = unspanned_lua_token_parser::table_constructor(tokens).unwrap();
        let table_constructor = TableConstructor::lfieldlist(exprs);
        assert_eq!(parsed, table_constructor);
        TestResult::passed()
    }

    #[quickcheck]
    fn parses_arbitrary_associative_table_constructor(
        exprs: Vec<(Ident, Expression)>,
    ) -> TestResult {
        if exprs.len() == 0 {
            return TestResult::discard();
        }
        let expected = TableConstructor::ffieldlist(exprs);
        let tokens: Vec<_> = expected.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::table_constructor(tokens).unwrap();
        assert_eq!(parsed, expected);
        TestResult::passed()
    }
}
