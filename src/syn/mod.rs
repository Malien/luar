use crate::lex::{Ident, Token};

pub mod expr;
pub use expr::op::*;

use expr::*;

#[cfg(test)]
pub mod arbitrary_tokens;

#[derive(Debug, PartialEq, Clone)]
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
    pub grammar lua_parser() for [Token] {
        pub rule nil() -> Expression
            = _:[Token::Nil] { Expression::Nil }

        pub rule string() -> Expression
            = _:[Token::String(literal)] { Expression::String(literal) }

        pub rule number() -> Expression
            = _:[Token::Number(literal)] { Expression::Number(literal) }

        pub rule var_expression() -> Expression
            = var:var() { Expression::Variable(var) }

        pub rule tbl_expression() -> Expression
            = tbl:table_constructor() { Expression::TableConstructor(tbl) }

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
            x:(@) _:[Token::Less] y:@ {
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
            x:(@) _:[Token::Mul] y:@ {
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
            e:tbl_expression() { e }
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

        pub rule table_constructor() -> TableConstructor
            = _:[Token::OpenSquigglyBracket] tc:table_constructor_contents() _:[Token::CloseSquigglyBracket] { tc }

        rule table_constructor_contents() -> TableConstructor
            = list:lfieldlist() {
                let mut list = list;
                list.reverse();
                TableConstructor::LFieldList(list)
            }
            / { TableConstructor::Empty }

        rule lfieldlist() -> Vec<Expression>
            = head:expression() tail:_lfieldlist_after_expr() {
                let mut tail = tail;
                tail.push(head);
                tail
            }

        rule _lfieldlist() -> Vec<Expression>
            = head:expression() tail:_lfieldlist_after_expr() {
                let mut tail = tail;
                tail.push(head);
                tail
            }
            / { Vec::new() }

        rule _lfieldlist_after_expr() -> Vec<Expression>
            = _:[Token::Comma] rest:_lfieldlist() { rest }
            / { Vec::new() }

    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use logos::Logos;

    use crate::lex::Token;

    use super::lua_parser;

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
