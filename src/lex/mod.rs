mod ident;
mod number_literal;
mod string_literal;
mod token;

pub type DynTokens = Box<dyn Iterator<Item = Token>>;

pub trait ToTokenStream {
    type Tokens: IntoIterator<Item = Token>;

    fn to_tokens(self) -> Self::Tokens;
}

#[macro_export]
macro_rules! fmt_tokens {
    ($type:ty) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                for token in self.clone().to_tokens() {
                    write!(fmt, "{} ", token)?;
                }
                Ok(())
            }
        }
    };
}

pub use ident::Ident;
pub use number_literal::NumberLiteral;
pub use string_literal::StringLiteral;
pub use token::Token;
