use crate::lex::{Ident, NumberLiteral, StringLiteral, Token};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOperator {
    // Precedence level 0
    And,
    Or,
    // Precedence level 1
    Less,
    Greater,
    LessOrEquals,
    GreaterOrEquals,
    NotEquals,
    Equals,
    // Precedence level 2
    Concat,
    // Precedence level 3
    Plus,
    Minus,
    // Precedence level 4
    Mul,
    Div,
    // Precedence level 5
    Exp,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOperator {
    Minus,
    Not,
}

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
        op: BinaryOperator,
        rhs: Box<Expression>,
    },
    UnaryOperator {
        op: UnaryOperator,
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

// macro_rules! l_binary_op_expr {
//     ($token:pat, $op:expr) => {
//         lhs:(@) _:[$token] rhs:@ {
//             Expression::BinaryOperator {
//                 op: $op,
//                 lhs: Box::new(lhs),
//                 rhs: Box::new(rhs),
//             }
//         }
//     };
// }

// macro_rules! r_binary_op_expr {
//     ($token:pat, $op:expr) => {
//         lhs:@ _:[$token] rhs:(@) {
//             Expression::BinaryOperator {
//                 op: $op,
//                 lhs: Box::new(lhs),
//                 rhs: Box::new(rhs),
//             }
//         }
//     };
// }

peg::parser! {
    grammar lua_parser() for [Token] {
        pub rule nil() -> Expression
            = _:[Token::Nil] { Expression::Nil }

        pub rule string() -> Expression
            = _:[Token::String(literal)] { Expression::String(literal) }

        pub rule number() -> Expression
            = _:[Token::Number(literal)] { Expression::Number(literal) }

        pub rule var_expression() -> Expression
            = var:var() { Expression::Variable(var) }

        pub rule expression() -> Expression = precedence! {
            x:(@) _:[Token::And] y:@ {
                Expression::BinaryOperator {
                    op: BinaryOperator::And,
                    lhs: Box::new(x),
                    rhs: Box::new(y),
                }
            }
            x:(@) _:[Token::Or] y:@ {
                Expression::BinaryOperator {
                    op: BinaryOperator::Or,
                    lhs: Box::new(x),
                    rhs: Box::new(y),
                }
            }
            --
            x:(@) _:[Token::Lesser] y:@ {
                Expression::BinaryOperator {
                    op: BinaryOperator::Less,
                    lhs: Box::new(x),
                    rhs: Box::new(y),
                }
            }
            x:(@) _:[Token::Greater] y:@ {
                Expression::BinaryOperator {
                    op: BinaryOperator::Greater,
                    lhs: Box::new(x),
                    rhs: Box::new(y),
                }
            }
            x:(@) _:[Token::LessOrEquals] y:@ {
                Expression::BinaryOperator {
                    op: BinaryOperator::LessOrEquals,
                    lhs: Box::new(x),
                    rhs: Box::new(y),
                }
            }
            x:(@) _:[Token::GreaterOrEquals] y:@ {
                Expression::BinaryOperator {
                    op: BinaryOperator::GreaterOrEquals,
                    lhs: Box::new(x),
                    rhs: Box::new(y),
                }
            }
            x:(@) _:[Token::NotEquals] y:@ {
                Expression::BinaryOperator {
                    op: BinaryOperator::NotEquals,
                    lhs: Box::new(x),
                    rhs: Box::new(y),
                }
            }
            x:(@) _:[Token::Equals] y:@ {
                Expression::BinaryOperator {
                    op: BinaryOperator::Equals,
                    lhs: Box::new(x),
                    rhs: Box::new(y),
                }
            }
            --
            x:(@) _:[Token::Concat] y:@ {
                Expression::BinaryOperator {
                    op: BinaryOperator::Concat,
                    lhs: Box::new(x),
                    rhs: Box::new(y),
                }
            }
            --
            x:(@) _:[Token::Plus] y:@ {
                Expression::BinaryOperator {
                    op: BinaryOperator::Plus,
                    lhs: Box::new(x),
                    rhs: Box::new(y),
                }
            }
            x:(@) _:[Token::Minus] y:@ {
                Expression::BinaryOperator {
                    op: BinaryOperator::Minus,
                    lhs: Box::new(x),
                    rhs: Box::new(y),
                }
            }
            --
            x:(@) _:[Token::Multiply] y:@ {
                Expression::BinaryOperator {
                    op: BinaryOperator::Mul,
                    lhs: Box::new(x),
                    rhs: Box::new(y),
                }
            }
            x:(@) _:[Token::Div] y:@ {
                Expression::BinaryOperator {
                    op: BinaryOperator::Div,
                    lhs: Box::new(x),
                    rhs: Box::new(y),
                }
            }
            --
            _:[Token::Not] x:@ {
                Expression::UnaryOperator {
                    op: UnaryOperator::Not,
                    exp: Box::new(x),
                }
            }
            _:[Token::Minus] x:@ {
                Expression::UnaryOperator {
                    op: UnaryOperator::Minus,
                    exp: Box::new(x),
                }
            }
            --
            x:@ _:[Token::Exp] y:(@) {
                Expression::BinaryOperator {
                    op: BinaryOperator::Exp,
                    lhs: Box::new(x),
                    rhs: Box::new(y),
                }
            }
            --
            e:nil() { e }
            e:string() { e }
            e:number() { e }
            e:var_expression() { e }
            _: [Token::OpenRoundBracket] e:expression() _:[Token::CloseRoundBracket] { e }
        }

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
    use indoc::indoc;
    use logos::Logos;
    use quickcheck::{Arbitrary, Gen};

    use super::{lua_parser, BinaryOperator, Expression, UnaryOperator, Var};
    use crate::lex::{Ident, NumberLiteral, StringLiteral, Token};

    use std::iter;

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

    impl Arbitrary for ArbitraryTokens<UnaryOperator> {
        fn arbitrary(g: &mut Gen) -> Self {
            match u8::arbitrary(g) % 2 {
                0 => (vec![Token::Minus], UnaryOperator::Minus),
                1 => (vec![Token::Not], UnaryOperator::Not),
                _ => unreachable!(),
            }
            .into()
        }
    }

    impl Arbitrary for ArbitraryTokens<BinaryOperator> {
        fn arbitrary(g: &mut Gen) -> Self {
            match u8::arbitrary(g) % 14 {
                0 => (vec![Token::And], BinaryOperator::And),
                1 => (vec![Token::Or], BinaryOperator::Or),
                2 => (vec![Token::Lesser], BinaryOperator::Less),
                3 => (vec![Token::Greater], BinaryOperator::Greater),
                4 => (vec![Token::LessOrEquals], BinaryOperator::LessOrEquals),
                5 => (
                    vec![Token::GreaterOrEquals],
                    BinaryOperator::GreaterOrEquals,
                ),
                6 => (vec![Token::NotEquals], BinaryOperator::NotEquals),
                7 => (vec![Token::Equals], BinaryOperator::Equals),
                8 => (vec![Token::Concat], BinaryOperator::Concat),
                9 => (vec![Token::Plus], BinaryOperator::Plus),
                10 => (vec![Token::Minus], BinaryOperator::Minus),
                11 => (vec![Token::Multiply], BinaryOperator::Mul),
                12 => (vec![Token::Div], BinaryOperator::Div),
                13 => (vec![Token::Exp], BinaryOperator::Exp),
                _ => unreachable!(),
            }
            .into()
        }
    }

    impl Arbitrary for ArbitraryTokens<Expression> {
        fn arbitrary(g: &mut Gen) -> Self {
            match u8::arbitrary(g) % 6 {
                0 => ArbitraryTokens {
                    tokens: vec![Token::Nil],
                    expected: Expression::Nil,
                },
                1 => {
                    let literal = NumberLiteral::arbitrary(g);
                    ArbitraryTokens {
                        tokens: vec![Token::Number(literal)],
                        expected: Expression::Number(literal),
                    }
                }
                2 => {
                    let literal = StringLiteral::arbitrary(g);
                    ArbitraryTokens {
                        tokens: vec![Token::String(literal.clone())],
                        expected: Expression::String(literal),
                    }
                }
                3 => {
                    let ArbitraryTokens { expected, tokens } = ArbitraryTokens::arbitrary(g);
                    ArbitraryTokens {
                        tokens,
                        expected: Expression::Variable(expected),
                    }
                }
                4 => {
                    let ArbitraryTokens {
                        expected: op,
                        tokens: op_tokens,
                    } = ArbitraryTokens::arbitrary(g);
                    let ArbitraryTokens {
                        expected: exp,
                        tokens: exp_tokens,
                    } = ArbitraryTokens::arbitrary(g);

                    let tokens: Vec<_> = iter::once(Token::OpenRoundBracket)
                        .chain(op_tokens)
                        .chain(exp_tokens)
                        .chain(iter::once(Token::CloseRoundBracket))
                        .collect();

                    ArbitraryTokens {
                        tokens,
                        expected: Expression::UnaryOperator {
                            op,
                            exp: Box::new(exp),
                        },
                    }
                }
                5 => {
                    let ArbitraryTokens {
                        expected: op,
                        tokens: op_tokens,
                    } = ArbitraryTokens::arbitrary(g);
                    let ArbitraryTokens {
                        expected: lhs,
                        tokens: lhs_tokens,
                    } = ArbitraryTokens::arbitrary(g);
                    let ArbitraryTokens {
                        expected: rhs,
                        tokens: rhs_tokens,
                    } = ArbitraryTokens::arbitrary(g);

                    let tokens: Vec<_> = iter::once(Token::OpenRoundBracket)
                        .chain(lhs_tokens)
                        .chain(op_tokens)
                        .chain(rhs_tokens)
                        .chain(iter::once(Token::CloseRoundBracket))
                        .collect();

                    ArbitraryTokens {
                        tokens,
                        expected: Expression::BinaryOperator {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                    }
                }
                _ => unreachable!(),
            }
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
    fn var_expr(ArbitraryTokens { tokens, expected }: ArbitraryTokens<Var>) {
        let parsed = lua_parser::expression(&tokens).unwrap();
        assert_eq!(parsed, Expression::Variable(expected));
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

    #[test]
    fn operator_precedence_1() {
        /*
        (
            (
                (
                    (
                        (
                            not (
                                (1)^(2)
                            )
                        ) * (
                            - (3)
                        )
                    ) + (
                        (4) / (5)
                    )
                ) .. (
                    (6) - (7)
                )
            ) < (8)
        ) and (
            (9) > (10)
        ) or (
            (11) == (12)
        )
        */
        static TEXT: &str = indoc! {"
            not 1^2 * - 3 + 4 / 5 .. 6 - 7 < 8 and 9 > 10 or 11 == 12
        "};
        let tokens: Vec<_> = Token::lexer(TEXT).collect();
        let expression = lua_parser::expression(&tokens).unwrap();
        insta::assert_debug_snapshot!(expression);
    }

    #[test]
    fn operator_precedence_2() {
        /*
        (
            (1) <= (2)
        ) and (
            (
                (3) + (
                    (
                        not (4)
                    ) * (
                        (5) ^ (6)
                    )
                )
            ) >= (
                (7) - (
                    (
                        - (8)
                    ) / (9)
                )
            )
        ) or (
            (10) ~= (
                (11) .. (12)
            )
        )
        */
        static TEXT: &str = indoc! {"
            1 <= 2 and 3 + not 4 * 5 ^ 6 >= 7 - - 8 / 9 or 10 ~= 11 .. 12
        "};
        let tokens: Vec<_> = Token::lexer(TEXT).collect();
        let expression = lua_parser::expression(&tokens).unwrap();
        insta::assert_debug_snapshot!(expression);
    }

    #[test]
    fn operator_precedence_3() {
        /*
        (
            (
                (1) <= (
                    (
                        (2) and (3)
                    ) + (
                        not (
                            (
                                (4) * (5)
                            )
                        ) ^ (6)
                    )
                )
            ) >= (
                (
                    (7) - (
                        (
                            - (8)
                        ) / (
                            (
                                (9) or (10)
                            ) ~= (11)
                        )
                    )
                ) .. (12)
            )
        )
        */
        static TEXT: &str = indoc! {"
            1 <= ((2 and 3) + not (4 * 5)) ^ 6 >= 7 - - 8 / ((9 or 10) ~= 11) .. 12
        "};
        let tokens: Vec<_> = Token::lexer(TEXT).collect();
        let expression = lua_parser::expression(&tokens).unwrap();
        insta::assert_debug_snapshot!(expression);
    }
}
