use crate::{
    fmt_tokens,
    lex::{DynTokens, Ident, ToTokenStream, Token},
    util::FlatIntersperseExt,
};

use super::{Expression, TableConstructor, Var};

use std::iter;

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionCall {
    Method {
        func: Var,
        method: Ident,
        args: FunctionCallArgs,
    },
    Function {
        func: Var,
        args: FunctionCallArgs,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionCallArgs {
    Table(TableConstructor),
    Arglist(Vec<Expression>),
}

fmt_tokens!(FunctionCall);

impl ToTokenStream for FunctionCall {
    type Tokens = DynTokens;

    fn to_tokens(self) -> Self::Tokens {
        match self {
            Self::Function { func, args } => Box::new(func.to_tokens().chain(args.to_tokens())),
            Self::Method { func, method, args } => Box::new(
                func.to_tokens()
                    .chain(iter::once(Token::Colon))
                    .chain(method.to_tokens())
                    .chain(args.to_tokens()),
            ),
        }
    }
}

impl ToTokenStream for FunctionCallArgs {
    type Tokens = DynTokens;

    fn to_tokens(self) -> Self::Tokens {
        match self {
            Self::Table(table) => table.to_tokens(),
            Self::Arglist(exprs) => Box::new(
                iter::once(Token::OpenRoundBracket)
                    .chain(
                        exprs
                            .into_iter()
                            .map(Expression::to_tokens)
                            .flat_intersperse(Token::Comma),
                    )
                    .chain(iter::once(Token::CloseRoundBracket)),
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use quickcheck::{empty_shrinker, Arbitrary, Gen};

    use crate::{lex::{Ident, ToTokenStream}, syn::{
            expr::{Expression, TableConstructor, Var},
            unspanned_lua_token_parser, ParseError,
        }, test_util::{QUICKCHECK_RECURSIVE_DEPTH, arbitrary_recursive_vec, with_thread_gen}};

    use super::{FunctionCall, FunctionCallArgs};

    impl Arbitrary for FunctionCall {
        fn arbitrary(g: &mut Gen) -> Self {
            match u8::arbitrary(g) % 2 {
                0 => Self::Method {
                    args: FunctionCallArgs::arbitrary(g),
                    func: Var::arbitrary(g),
                    method: with_thread_gen(Ident::arbitrary),
                },
                1 => Self::Function {
                    args: FunctionCallArgs::arbitrary(g),
                    func: Var::arbitrary(g),
                },
                _ => unreachable!(),
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match self {
                Self::Method { args, func, method } => {
                    let args_shrinked = {
                        let func = func.clone();
                        let method = method.clone();

                        args.shrink().map(move |args| Self::Method {
                            args,
                            func: func.clone(),
                            method: method.clone(),
                        })
                    };

                    let func_shrinked = {
                        let args = args.clone();
                        let method = method.clone();

                        func.shrink().map(move |func| Self::Method {
                            args: args.clone(),
                            func,
                            method: method.clone(),
                        })
                    };

                    let method_shrinked = {
                        let args = args.clone();
                        let func = func.clone();

                        method.shrink().map(move |method| Self::Method {
                            args: args.clone(),
                            func: func.clone(),
                            method,
                        })
                    };

                    Box::new(args_shrinked.chain(func_shrinked).chain(method_shrinked))
                }

                Self::Function { args, func } => {
                    let args_shrinked = {
                        let func = func.clone();
                        args.shrink().map(move |args| Self::Function {
                            args,
                            func: func.clone(),
                        })
                    };

                    let func_shrinked = {
                        let args = args.clone();

                        func.shrink().map(move |func| Self::Function {
                            args: args.clone(),
                            func,
                        })
                    };

                    Box::new(args_shrinked.chain(func_shrinked))
                }
            }
        }
    }

    impl Arbitrary for FunctionCallArgs {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            if g.size() == 0 {
                match u8::arbitrary(g) % 2 {
                    0 => Self::Table(TableConstructor::empty()),
                    1 => Self::Arglist(Vec::new()),
                    _ => unreachable!(),
                }
            } else {
                let gen = &mut Gen::new(QUICKCHECK_RECURSIVE_DEPTH.min(g.size() - 1));
                match u8::arbitrary(gen) % 2 {
                    0 => Self::Table(TableConstructor::arbitrary(g)),
                    1 => Self::Arglist(arbitrary_recursive_vec(gen)),
                    _ => unreachable!(),
                }
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match self {
                Self::Table(tbl) if tbl.is_empty() => empty_shrinker(),
                Self::Arglist(args) if args.is_empty() => empty_shrinker(),
                Self::Table(tbl) => Box::new(tbl.shrink().map(Self::Table)),
                Self::Arglist(args) => Box::new(args.shrink().map(Self::Arglist)),
            }
        }
    }

    #[quickcheck]
    fn parses_empty_function_call(func: Var) {
        let expected = FunctionCall::Function {
            func,
            args: FunctionCallArgs::Arglist(Vec::new()),
        };
        let tokens: Vec<_> = expected.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::function_call(tokens).unwrap();
        assert_eq!(expected, parsed);
    }

    #[quickcheck]
    fn parses_empty_table_function_cal(func: Var) {
        let expected = FunctionCall::Function {
            func,
            args: FunctionCallArgs::Table(TableConstructor::empty()),
        };
        let tokens: Vec<_> = expected.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::function_call(tokens).unwrap();
        assert_eq!(expected, parsed);
    }

    #[quickcheck]
    fn parses_arbitrary_table_function_call(func: Var, tbl: TableConstructor) {
        let expected = FunctionCall::Function {
            func,
            args: FunctionCallArgs::Table(tbl),
        };
        let tokens: Vec<_> = expected.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::function_call(tokens).unwrap();
        assert_eq!(expected, parsed);
    }

    #[quickcheck]
    fn parses_arbitrary_arglist_function_call(
        func: Var,
        args: Vec<Expression>,
    ) -> Result<(), ParseError> {
        let expected = FunctionCall::Function {
            func,
            args: FunctionCallArgs::Arglist(args),
        };
        let tokens: Vec<_> = expected.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::function_call(tokens)?;
        assert_eq!(expected, parsed);
        Ok(())
    }

    #[quickcheck]
    fn parses_arbitrary_function_function_call(
        func: Var,
        args: FunctionCallArgs,
    ) -> Result<(), ParseError> {
        let expected = FunctionCall::Function { func, args };
        let tokens: Vec<_> = expected.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::function_call(tokens)?;
        assert_eq!(expected, parsed);
        Ok(())
    }

    #[quickcheck]
    fn parses_arbitrary_method_call(
        func: Var,
        method: Ident,
        args: FunctionCallArgs,
    ) -> Result<(), ParseError> {
        let expected = FunctionCall::Method { func, method, args };
        let tokens: Vec<_> = expected.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::function_call(tokens)?;
        assert_eq!(expected, parsed);
        Ok(())
    }

    #[quickcheck]
    fn parses_arbitrary_function_call(expected: FunctionCall) -> Result<(), ParseError> {
        let tokens: Vec<_> = expected.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::function_call(tokens)?;
        assert_eq!(expected, parsed);
        Ok(())
    }
}
