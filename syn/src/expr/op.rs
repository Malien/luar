use std::iter;

use luar_lex::{fmt_tokens, ToTokenStream, Token};

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

impl ToTokenStream for BinaryOperator {
    type Tokens = iter::Once<Token>;
    fn to_tokens(self) -> Self::Tokens {
        iter::once(match self {
            BinaryOperator::And => Token::And,
            BinaryOperator::Or => Token::Or,
            BinaryOperator::Less => Token::Less,
            BinaryOperator::Greater => Token::Greater,
            BinaryOperator::LessOrEquals => Token::LessOrEquals,
            BinaryOperator::GreaterOrEquals => Token::GreaterOrEquals,
            BinaryOperator::NotEquals => Token::NotEquals,
            BinaryOperator::Equals => Token::Equals,
            BinaryOperator::Concat => Token::Concat,
            BinaryOperator::Plus => Token::Plus,
            BinaryOperator::Minus => Token::Minus,
            BinaryOperator::Mul => Token::Mul,
            BinaryOperator::Div => Token::Div,
            BinaryOperator::Exp => Token::Exp,
        })
    }
}

impl ToTokenStream for UnaryOperator {
    type Tokens = iter::Once<Token>;
    fn to_tokens(self) -> Self::Tokens {
        iter::once(match self {
            UnaryOperator::Minus => Token::Minus,
            UnaryOperator::Not => Token::Not,
        })
    }
}

fmt_tokens!(BinaryOperator);
fmt_tokens!(UnaryOperator);

#[cfg(feature = "quickcheck")]
use quickcheck::{Arbitrary, Gen};

#[cfg(feature = "quickcheck")]
impl Arbitrary for UnaryOperator {
    fn arbitrary(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 2 {
            0 => UnaryOperator::Minus,
            1 => UnaryOperator::Not,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "quickcheck")]
impl Arbitrary for BinaryOperator {
    fn arbitrary(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 14 {
            0 => BinaryOperator::And,
            1 => BinaryOperator::Or,
            2 => BinaryOperator::Less,
            3 => BinaryOperator::Greater,
            4 => BinaryOperator::LessOrEquals,
            5 => BinaryOperator::GreaterOrEquals,
            6 => BinaryOperator::NotEquals,
            7 => BinaryOperator::Equals,
            8 => BinaryOperator::Concat,
            9 => BinaryOperator::Plus,
            10 => BinaryOperator::Minus,
            11 => BinaryOperator::Mul,
            12 => BinaryOperator::Div,
            13 => BinaryOperator::Exp,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::unspanned_lua_token_parser;

    use indoc::indoc;
    use logos::Logos;
    use luar_lex::Token;

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
        let expression = unspanned_lua_token_parser::expression(tokens).unwrap();
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
        let expression = unspanned_lua_token_parser::expression(tokens).unwrap();
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
        let expression = unspanned_lua_token_parser::expression(tokens).unwrap();
        insta::assert_debug_snapshot!(expression);
    }
}
