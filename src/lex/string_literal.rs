use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringLiteral(String);

#[derive(Error, Debug)]
#[error("Token passed in is not a valid string literal")]
pub struct StringLiteralParseError;

impl FromStr for StringLiteral {
    type Err = StringLiteralParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 2 {
            return Err(StringLiteralParseError);
        }
        let raw_string: &str = &s[1..s.len() - 1];
        // Not a very efficient method to escape characters
        Ok(StringLiteral(
            raw_string
                .replace(r"\\", "\\")
                .replace(r"\r", "\r")
                .replace(r"\n", "\n")
                .replace(r"\t", "\t")
                .replace("\\\"", "\"")
                .replace(r"\'", "'"),
        ))
    }
}
// fn parse_string_literal(input: &mut Lexer<Token>) -> Result<String, StringLiteralParseError> {
//     let slice = input.slice();
// }
