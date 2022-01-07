use lazy_static::lazy_static;
#[cfg(test)]
use quickcheck::Arbitrary;
use regex::Regex;
use std::{
    iter::{self, FlatMap, Once},
    str::FromStr,
};
use thiserror::Error;

use super::{ToTokenStream, Token};

lazy_static! {
    static ref IDENT_REGEX: Regex = Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*$").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ident(String);

#[derive(Debug, Error)]
#[error("Invalid identifier")]
pub struct InvalidIdentifier;

impl FromStr for Ident {
    type Err = InvalidIdentifier;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if IDENT_REGEX.is_match(s) {
            Ok(Ident(s.to_string()))
        } else {
            Err(InvalidIdentifier)
        }
    }
}

impl Ident {
    #[allow(dead_code)]
    pub fn try_new(str: String) -> Option<Self> {
        if IDENT_REGEX.is_match(&str) {
            Some(Ident(str))
        } else {
            None
        }
    }

    pub fn new(str: impl Into<String>) -> Self {
        Ident(str.into())
    }
}

impl ToTokenStream for Ident {
    type Tokens = iter::Once<Token>;
    fn to_tokens(self) -> Self::Tokens {
        iter::once(Token::Ident(self))
    }
}

impl From<Ident> for String {
    fn from(Ident(str): Ident) -> Self {
        str
    }
}

impl<'a> From<&'a Ident> for &'a String {
    fn from(Ident(str): &'a Ident) -> Self {
        str
    }
}

impl<'a> AsRef<str> for &'a Ident {
    fn as_ref(&self) -> &'a str {
        self.0.as_ref()
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
const VALID_IDENT_CHARS: &'static str =
    "1234567890_abcdefghigklmnopqrstuvwxyzABCDEFGHIGKLMNOPQRSTUVWXYZ";
#[cfg(test)]
const VALID_IDENT_BYTES: &[u8] = VALID_IDENT_CHARS.as_bytes();

#[cfg(test)]
impl Arbitrary for Ident {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if g.size() == 0 {
            return Ident::new("_");
        }
        let mut buf = Vec::with_capacity(g.size());
        let beginning_bytes = &VALID_IDENT_BYTES[10..];
        // Could use unsafe version unwrap_unchecked()
        buf.push(*g.choose(beginning_bytes).unwrap());
        for _ in 1..g.size() {
            buf.push(*g.choose(VALID_IDENT_BYTES).unwrap());
        }
        // Could use unsafe version from_utf8_unchecked(Vec<u8>)
        let str = String::from_utf8(buf).unwrap();
        Self(str)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().filter_map(Ident::try_new))
    }
}

impl ToTokenStream for Vec<Ident> {
    // I hate how this is forcing FlatMap to take a function pointer
    // rather than be specialized with exact function type. Gimme impl
    // Iterator, or at least let me name types of specific functions!
    type Tokens = FlatMap<
        std::vec::IntoIter<Ident>,
        Once<Token>,
        fn(Ident) -> <Ident as ToTokenStream>::Tokens,
    >;

    fn to_tokens(self) -> Self::Tokens {
        self.into_iter().flat_map(ToTokenStream::to_tokens)
    }
}
