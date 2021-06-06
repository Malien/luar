use std::str::FromStr;
use regex::Regex;
use thiserror::Error;
use lazy_static::lazy_static;

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
    pub fn try_new(str: &str) -> Result<Self, InvalidIdentifier> {
        str.parse()
    }

    pub unsafe fn from_raw(str: &str) -> Self {
        Ident(str.to_string())
    }
}