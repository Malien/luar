use std::iter;

use super::Expression;
use crate::lex::DynTokens;
use crate::lex::Ident;
use crate::lex::ToTokenStream;
use crate::lex::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Var {
    Named(Ident),
    PropertyAccess {
        from: Box<Var>,
        property: Ident,
    },
    MemberLookup {
        from: Box<Var>,
        value: Box<Expression>,
    },
}

impl ToTokenStream for Var {
    type Tokens = DynTokens;
    fn to_tokens(self) -> Self::Tokens {
        match self {
            Var::Named(ident) => Box::new(IntoIterator::into_iter(ident.to_tokens())),
            Var::PropertyAccess { from, property } => Box::new(
                from.to_tokens()
                    .chain(iter::once(Token::Dot))
                    .chain(property.to_tokens()),
            ),
            Var::MemberLookup { from, value } => Box::new(
                from.to_tokens()
                    .chain(iter::once(Token::OpenSquareBracket))
                    .chain(value.to_tokens())
                    .chain(iter::once(Token::CloseSquareBracket)),
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use quickcheck::{empty_shrinker, Arbitrary, Gen};
    use std::iter;

    use crate::{
        lex::{Ident, ToTokenStream, Token},
        syn::{
            expr::{Expression, Var},
            unspanned_lua_token_parser, RawParseError,
        },
        test_util::{with_thread_gen, QUICKCHECK_RECURSIVE_DEPTH},
    };

    impl Arbitrary for Var {
        fn arbitrary(g: &mut Gen) -> Self {
            if g.size() == 0 {
                Var::Named(with_thread_gen(Ident::arbitrary))
            } else {
                let g = &mut Gen::new(QUICKCHECK_RECURSIVE_DEPTH.min(g.size() - 1));
                match u8::arbitrary(g) % 2 {
                    0 => Var::PropertyAccess {
                        from: Box::new(Var::arbitrary(g)),
                        property: with_thread_gen(Ident::arbitrary),
                    },
                    1 => Var::MemberLookup {
                        from: Box::new(Var::arbitrary(g)),
                        value: Box::new(Expression::arbitrary(g)),
                    },
                    _ => unreachable!(),
                }
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match self {
                Var::Named(_) => empty_shrinker(),
                Var::PropertyAccess { from, .. } => Box::new(iter::once(from.as_ref().clone())),
                // Var::MemberLookup { from, .. } => Box::new(iter::once(from.as_ref().clone())),
                Var::MemberLookup { from, value } => {
                    let from = Box::clone(from);
                    Box::new(iter::once(from.as_ref().clone()).chain(value.shrink().map(
                        move |expr| Var::MemberLookup {
                            from: Box::clone(&from),
                            value: expr,
                        },
                    )))
                }
            }
        }
    }

    #[quickcheck]
    fn parse_named_var(ident: Ident) {
        let parsed = unspanned_lua_token_parser::var([Token::Ident(ident.clone())]).unwrap();
        assert_eq!(Var::Named(ident), parsed);
    }

    #[quickcheck]
    fn parse_single_ppty_access(base: Ident, property: Ident) {
        let parsed = unspanned_lua_token_parser::var([
            Token::Ident(base.clone()),
            Token::Dot,
            Token::Ident(property.clone()),
        ])
        .unwrap();
        assert_eq!(
            Var::PropertyAccess {
                from: Box::new(Var::Named(base)),
                property: property
            },
            parsed
        );
    }

    #[quickcheck]
    fn parse_arbitrary_ppty_access(base: Ident, properties: Vec<Ident>) {
        let mut sequence = Vec::with_capacity(properties.len() + 1);
        sequence.push(Token::Ident(base.clone()));
        sequence.extend(properties.iter().cloned().flat_map(|property| {
            std::iter::once(Token::Dot).chain(std::iter::once(Token::Ident(property)))
        }));
        let parsed = unspanned_lua_token_parser::var(sequence).unwrap();
        let mut var = Var::Named(base);
        for property in properties {
            var = Var::PropertyAccess {
                from: Box::new(var),
                property,
            }
        }
        assert_eq!(var, parsed);
    }

    #[quickcheck]
    fn parse_single_member_lookup(base: Ident, expression: Expression) -> Result<(), RawParseError>{
        let mut tokens = vec![Token::Ident(base.clone()), Token::OpenSquareBracket];
        tokens.extend(expression.clone().to_tokens());
        tokens.push(Token::CloseSquareBracket);
        let parsed = unspanned_lua_token_parser::var(tokens)?;
        assert_eq!(
            parsed,
            Var::MemberLookup {
                from: Box::new(Var::Named(base)),
                value: Box::new(expression)
            }
        );
        Ok(())
    }

    #[quickcheck]
    fn parse_arbitrary_member_lookup(base: Ident, expressions: Vec<Expression>) {
        let mut sequence = Vec::with_capacity(expressions.len() * 3 + 1);
        sequence.push(Token::Ident(base.clone()));
        sequence.extend(expressions.iter().flat_map(|expression| {
            let mut tokens = Vec::new();
            tokens.push(Token::OpenSquareBracket);
            tokens.extend(expression.clone().to_tokens());
            tokens.push(Token::CloseSquareBracket);
            tokens
        }));
        let parsed = unspanned_lua_token_parser::var(sequence).unwrap();
        let mut var = Var::Named(base);
        for expression in expressions {
            var = Var::MemberLookup {
                from: Box::new(var),
                value: Box::new(expression),
            }
        }
        assert_eq!(var, parsed);
    }

    #[quickcheck]
    fn parse_arbitrary_var(expected: Var) {
        let tokens = expected.clone().to_tokens().collect::<Vec<_>>();
        let parsed = unspanned_lua_token_parser::var(tokens).unwrap();
        assert_eq!(parsed, expected);
    }
}
