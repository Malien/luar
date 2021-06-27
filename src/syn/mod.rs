use crate::lex::{Ident, NumberLiteral, StringLiteral, Token};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operator {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TableConstructor {}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Nil,
    String(StringLiteral),
    Number(NumberLiteral),
    Variable(Var),
    BinaryOperator {
        lhs: Box<Expression>,
        op: Operator,
        rhs: Box<Expression>,
    },
    UnaryOperator {
        op: Operator,
        exp: Box<Expression>,
    },
    TableConstructor(TableConstructor),
    FunctionCall {
        func: Var,
        args: Vec<Expression>,
    },
}

#[derive(Debug, PartialEq, Clone)]
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

enum VarLeftover {
    Nothing,
    PropertyAccess {
        from: Box<VarLeftover>,
        property: Ident,
    },
    MemberLookup {
        from: Box<VarLeftover>,
        value: Expression,
    },
}

fn accumulate_var_leftovers(base: Var, leftovers: VarLeftover) -> Var {
    match leftovers {
        VarLeftover::Nothing => base,
        VarLeftover::PropertyAccess { from, property } => accumulate_var_leftovers(
            Var::PropertyAccess {
                from: Box::new(base),
                property,
            },
            *from,
        ),
        VarLeftover::MemberLookup { from, value } => accumulate_var_leftovers(
            Var::MemberLookup {
                from: Box::new(base),
                value: Box::new(value),
            },
            *from,
        ),
    }
}

peg::parser! {
    grammar lua_parser() for [Token] {
        pub rule nil() -> Expression
            = _:[Token::Nil] { Expression::Nil }

        pub rule string() -> Expression
            = _:[Token::String(literal)] { Expression::String(literal) }

        pub rule number() -> Expression
            = _:[Token::Number(literal)] { Expression::Number(literal) }

        pub rule expression() -> Expression
            = e:nil() { e }
            / e:string() { e }
            / e:number() { e }

        pub rule named() -> Var
            = _:[Token::Ident(ident)] { Var::Named(ident) }

        pub rule property_access() -> Var
            = base:var() _:[Token::Dot] _:[Token::Ident(property) ] {
                Var::PropertyAccess {
                    from: Box::new(base),
                    property
                }
            }

        pub rule var() -> Var
            = base:named() leftovers:_var() { accumulate_var_leftovers(base, leftovers) }

        rule _var() -> VarLeftover
            = _:[Token::Dot] _:[Token::Ident(ident)] next:_var() {
                VarLeftover::PropertyAccess {
                    from: Box::new(next),
                    property: ident
                }
            }
            / _:[Token::OpenSquareBracket] e:expression() _:[Token::CloseSquareBracket] next:_var() {
                VarLeftover::MemberLookup {
                    from: Box::new(next),
                    value: e
                }
            }
            / { VarLeftover::Nothing }
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::{Arbitrary, Gen};

    use super::{lua_parser, Expression, Var};
    use crate::lex::{Ident, NumberLiteral, StringLiteral, Token};

    #[derive(Debug, Clone)]
    struct ArbitraryTokens<T> {
        tokens: Vec<Token>,
        expected: T,
    }

    impl<T> From<(Vec<Token>, T)> for ArbitraryTokens<T> {
        fn from((tokens, expected): (Vec<Token>, T)) -> Self {
            Self { tokens, expected }
        }
    }

    impl Arbitrary for ArbitraryTokens<Expression> {
        fn arbitrary(g: &mut Gen) -> Self {
            match u8::arbitrary(g) % 3 {
                0 => (vec![Token::Nil], Expression::Nil),
                1 => {
                    let literal = NumberLiteral::arbitrary(g);
                    (vec![Token::Number(literal)], Expression::Number(literal))
                }
                2 => {
                    let literal = StringLiteral::arbitrary(g);
                    (
                        vec![Token::String(literal.clone())],
                        Expression::String(literal),
                    )
                }
                _ => unreachable!(),
            }
            .into()
        }
    }

    #[test]
    fn nill_expr() {
        let parsed = lua_parser::expression(&[Token::Nil]).unwrap();
        assert_eq!(Expression::Nil, parsed);
    }

    #[quickcheck]
    fn number_expr(literal: NumberLiteral) {
        let expression = lua_parser::expression(&[Token::Number(literal)]).unwrap();
        match (&literal, &expression) {
            (NumberLiteral(x), Expression::Number(NumberLiteral(y))) if f64::is_nan(*x) => {
                assert!(f64::is_nan(*y))
            }
            _ => assert_eq!(Expression::Number(literal), expression),
        };
    }

    #[quickcheck]
    fn string_expr(literal: StringLiteral) {
        assert_eq!(
            Expression::String(literal.clone()),
            lua_parser::expression(&[Token::String(literal)]).unwrap()
        );
    }

    #[quickcheck]
    fn parses_arbitrary_expression(
        ArbitraryTokens { tokens, expected }: ArbitraryTokens<Expression>,
    ) {
        let parsed = lua_parser::expression(&tokens).unwrap();
        assert_eq!(parsed, expected);
    }

    impl Arbitrary for ArbitraryTokens<Var> {
        fn arbitrary(g: &mut Gen) -> Self {
            if g.size() <= 1 {
                let ident = Ident::arbitrary(g);
                return (vec![Token::Ident(ident.clone())], Var::Named(ident)).into();
            }
            match u8::arbitrary(g) % 3 {
                0 => {
                    let ident = Ident::arbitrary(g);
                    (vec![Token::Ident(ident.clone())], Var::Named(ident)).into()
                }
                1 => {
                    let ident = Ident::arbitrary(g);
                    let ArbitraryTokens {
                        mut tokens,
                        expected,
                    } = ArbitraryTokens::arbitrary(g);
                    tokens.push(Token::Dot);
                    tokens.push(Token::Ident(ident.clone()));
                    (
                        tokens,
                        Var::PropertyAccess {
                            from: Box::new(expected),
                            property: ident,
                        },
                    )
                }
                2 => {
                    let expression = ArbitraryTokens::arbitrary(g);
                    let ArbitraryTokens {
                        mut tokens,
                        expected,
                    } = ArbitraryTokens::arbitrary(g);
                    tokens.push(Token::OpenSquareBracket);
                    tokens.extend(expression.tokens);
                    tokens.push(Token::CloseSquareBracket);
                    (
                        tokens,
                        Var::MemberLookup {
                            from: Box::new(expected),
                            value: Box::new(expression.expected),
                        },
                    )
                }
                _ => unreachable!(),
            }
            .into()
        }
    }

    #[quickcheck]
    fn parse_named_var(ident: Ident) {
        let parsed = lua_parser::var(&[Token::Ident(ident.clone())]).unwrap();
        assert_eq!(Var::Named(ident), parsed);
    }

    #[quickcheck]
    fn parse_single_ppty_access(base: Ident, property: Ident) {
        let parsed = lua_parser::var(&[
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
        let parsed = lua_parser::var(&sequence).unwrap();
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
    fn parse_single_member_lookup(base: Ident, expression: ArbitraryTokens<Expression>) {
        let mut tokens = vec![Token::Ident(base.clone()), Token::OpenSquareBracket];
        tokens.extend(expression.tokens);
        tokens.push(Token::CloseSquareBracket);
        let parsed = lua_parser::var(&tokens).unwrap();
        assert_eq!(
            parsed,
            Var::MemberLookup {
                from: Box::new(Var::Named(base)),
                value: Box::new(expression.expected)
            }
        );
    }

    #[quickcheck]
    fn parse_arbitrary_member_lookup(base: Ident, expressions: Vec<ArbitraryTokens<Expression>>) {
        let mut sequence = Vec::with_capacity(expressions.len() * 3 + 1);
        sequence.push(Token::Ident(base.clone()));
        sequence.extend(expressions.iter().flat_map(|expression| {
            let mut tokens = Vec::with_capacity(expression.tokens.len() + 3);
            tokens.push(Token::OpenSquareBracket);
            tokens.extend_from_slice(&expression.tokens);
            tokens.push(Token::CloseSquareBracket);
            tokens
        }));
        let parsed = lua_parser::var(&sequence).unwrap();
        let mut var = Var::Named(base);
        for expression in expressions {
            var = Var::MemberLookup {
                from: Box::new(var),
                value: Box::new(expression.expected),
            }
        }
        assert_eq!(var, parsed);
    }

    #[quickcheck]
    fn parse_arbitrary_var(ArbitraryTokens { tokens, expected }: ArbitraryTokens<Var>) {
        let parsed = lua_parser::var(&tokens).unwrap();
        assert_eq!(parsed, expected);
    }
}
