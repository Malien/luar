mod ident;
mod number_literal;
mod string_literal;
mod token;
pub mod format;

pub type DynTokens = Box<dyn Iterator<Item = Token>>;

pub trait ToTokenStream {
    type Tokens: IntoIterator<Item = Token>;

    fn to_tokens(self) -> Self::Tokens;
}

pub use ident::Ident;
pub use number_literal::NumberLiteral;
pub use string_literal::StringLiteral;
pub use token::Token;
