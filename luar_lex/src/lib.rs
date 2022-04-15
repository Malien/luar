#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod format;
mod ident;
mod number_literal;
mod string_literal;
mod token;

pub type DynTokens = Box<dyn Iterator<Item = Token>>;

pub trait ToTokenStream {
    type Tokens: IntoIterator<Item = Token>;

    fn to_tokens(self) -> Self::Tokens;
}

impl<T: IntoIterator<Item = Token>> ToTokenStream for T {
    type Tokens = Self;

    fn to_tokens(self) -> Self::Tokens {
        self
    }
}

pub use ident::Ident;
pub use number_literal::NumberLiteral;
pub use string_literal::StringLiteral;
pub use token::Token;
