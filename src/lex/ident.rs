use lazy_static::lazy_static;
#[cfg(test)]
use quickcheck::Arbitrary;
use regex::Regex;
use std::{iter, str::FromStr};
use thiserror::Error;

use super::{ToTokenStream, Token};

lazy_static! {
    static ref IDENT_REGEX: Regex = Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*$").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

    pub unsafe fn new_unchecked(str: &str) -> Self {
        Ident(str.to_string())
    }
}

impl ToTokenStream for Ident {
    type Tokens = iter::Once<Token>;
    fn to_tokens(self) -> Self::Tokens {
        iter::once(Token::Ident(self))
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
impl Arbitrary for Ident {
    fn arbitrary(_: &mut quickcheck::Gen) -> Self {
        unsafe { Ident::new_unchecked("foo") }
        // loop {
        //     if let Some(ident) = Ident::try_new(String::arbitrary(g)) {
        //         return ident;
        //     }
        // }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().filter_map(Ident::try_new))
    }
}
