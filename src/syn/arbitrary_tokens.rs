use crate::lex::Token;

#[derive(Debug, Clone)]
pub struct ArbitraryTokens<T> {
    pub tokens: Vec<Token>,
    pub expected: T,
}

impl<T> From<(Vec<Token>, T)> for ArbitraryTokens<T> {
    fn from((tokens, expected): (Vec<Token>, T)) -> Self {
        Self { tokens, expected }
    }
}
