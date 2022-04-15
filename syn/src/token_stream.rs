use std::iter::FromIterator;

use peg::{Parse, ParseElem};

use luar_lex::{ToTokenStream, Token};

use super::TokenSpan;

pub struct TokenStream(Vec<(Token, TokenSpan)>);

impl FromIterator<(Token, std::ops::Range<usize>)> for TokenStream {
    fn from_iter<T: IntoIterator<Item = (Token, std::ops::Range<usize>)>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|(token, span)| (token, span.into()))
                .collect(),
        )
    }
}

impl FromIterator<Token> for TokenStream {
    fn from_iter<T: IntoIterator<Item = Token>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .zip(0..)
                .map(|(token, position)| (token, TokenSpan::StreamPosition(position)))
                .collect(),
        )
    }
}

pub trait ToTokenStreamExt: ToTokenStream {
    fn to_spanned_token_stream(self) -> TokenStream
    where
        Self: Sized,
    {
        self.to_tokens().into_iter().collect()
    }
}

impl<T: ToTokenStream> ToTokenStreamExt for T {}

impl Parse for TokenStream {
    type PositionRepr = TokenSpan;

    fn start<'input>(&'input self) -> usize {
        0
    }

    fn is_eof<'input>(&'input self, p: usize) -> bool {
        p >= self.0.len()
    }

    fn position_repr<'input>(&'input self, p: usize) -> Self::PositionRepr {
        match self.0.get(p) {
            Some((_, pos)) => *pos,
            None => TokenSpan::Unknown,
        }
    }
}

impl ParseElem for TokenStream {
    type Element = Token;

    fn parse_elem(&self, pos: usize) -> peg::RuleResult<Self::Element> {
        match self.0.get(pos) {
            Some((token, _)) => peg::RuleResult::Matched(pos + 1, token.clone()),
            None => peg::RuleResult::Failed,
        }
    }
}
