#[cfg(test)]
use quickcheck::{Arbitrary, Gen};
use std::{iter, str::FromStr};
use thiserror::Error;

use super::{ToTokenStream, Token};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringLiteral(pub String);

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

#[cfg(test)]
impl Arbitrary for StringLiteral {
    fn arbitrary(g: &mut Gen) -> Self {
        StringLiteral(String::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(StringLiteral))
    }
}

impl ToTokenStream for StringLiteral {
    type Tokens = iter::Once<Token>;

    fn to_tokens(self) -> Self::Tokens {
        iter::once(Token::String(self))
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::TestResult;

    use super::StringLiteral;

    #[quickcheck]
    fn parses_plain_string(input: String) -> TestResult {
        if input.chars().any(|c| !c.is_alphanumeric()) {
            return TestResult::discard();
        }

        let StringLiteral(res) = format!("\"{}\"", input).parse().unwrap();
        assert_eq!(res, input);

        TestResult::passed()
    }

    #[test]
    fn parses_escape_sequences() {
        let StringLiteral(res) = r"'hello \n\r\t\\\'world\''".parse().unwrap();
        assert_eq!(res, "hello \n\r\t\\'world'");
    }
}
