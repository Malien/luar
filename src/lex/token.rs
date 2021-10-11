use logos::Logos;

use super::{Ident, NumberLiteral, StringLiteral};

/// Reserved words:
///     `and` `do` `else` `elseif` `end` `function` `if` `local` `nil` `not`
///     `or` `repeat` `return` `until` `then` `while`
///
/// Other tokens:
///     `==` `~=` `<=` `>=` `<` `>` `=` `..` `+` `-` `*` `/` `%` `(` `)` `{` `}` `[` `]` `:` `;` `,` `.`
///
/// Comments denoted by `--` and continue until the end of the line
///
/// Strings are delimited with either single or double quotes, and can contain C-like escape sequences (like `\n` `\r` `\t`)
///
/// Numbers are have either
///     `4` `4.23` `4.` `.23` `4.57e-7` `.3e4`

#[derive(Clone, Debug, PartialEq, Logos)]
pub enum Token {
    #[error]
    #[regex(r"[ \t\n\f]", logos::skip)]
    #[regex("--.*", logos::skip)]
    Error,
    #[token("and")]
    And,
    #[token("do")]
    Do,
    #[token("else")]
    Else,
    #[token("elseif")]
    ElseIf,
    #[token("end")]
    End,
    #[token("function")]
    Function,
    #[token("if")]
    If,
    #[token("local")]
    Local,
    #[token("nil")]
    Nil,
    #[token("not")]
    Not,
    #[token("or")]
    Or,
    #[token("repeat")]
    Repeat,
    #[token("return")]
    Return,
    #[token("until")]
    Until,
    #[token("then")]
    Then,
    #[token("while")]
    While,
    #[token("==")]
    Equals,
    #[token("~=")]
    NotEquals,
    #[token("<=")]
    LessOrEquals,
    #[token(">=")]
    GreaterOrEquals,
    #[token(">")]
    Greater,
    #[token("<")]
    Less,
    #[token("=")]
    Assignment,
    #[token("..")]
    Concat,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("%")]
    Mod,
    #[token("^")]
    Exp,
    #[token("(")]
    OpenRoundBracket,
    #[token(")")]
    CloseRoundBracket,
    #[token("[")]
    OpenSquareBracket,
    #[token("]")]
    CloseSquareBracket,
    #[token("{")]
    OpenSquigglyBracket,
    #[token("}")]
    CloseSquigglyBracket,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    // SAFETY: This is the same regex used to check for identifier validity
    #[regex(r"[_a-zA-Z][_a-zA-Z0-9]*", |str| unsafe { Ident::from_raw(str.slice()) })]
    Ident(Ident),
    #[regex(
        "(\"(?:[^\"'\\\\]|\\\\.)*\")|('(?:[^\"'\\\\]|\\\\.)*')",
        |token| token.slice().parse()
    )]
    String(StringLiteral),
    // TODO: Remove unary + or -. Parse NaN and Inf
    #[regex(
        r"[+-]?((\d+\.\d+)|(\.\d+)|(\d+\.?))(e[+-]?\d+)?",
        |token| token.slice().parse()
    )]
    Number(NumberLiteral),
}

impl Token {
    pub fn is_err(&self) -> bool {
        if let Token::Error = self {
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::unreachable;

    use crate::lex::{Ident, NumberLiteral, StringLiteral};

    use super::Token;
    use logos::Logos;
    use quickcheck::Arbitrary;

    impl Arbitrary for Token {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let idx = u8::arbitrary(g) % 44;
            match idx {
                0 => Token::And,
                1 => Token::Do,
                3 => Token::Else,
                4 => Token::ElseIf,
                5 => Token::End,
                6 => Token::Function,
                7 => Token::If,
                8 => Token::Local,
                9 => Token::Nil,
                10 => Token::Not,
                11 => Token::Or,
                12 => Token::Repeat,
                13 => Token::Return,
                14 => Token::Until,
                15 => Token::Then,
                16 => Token::While,
                17 => Token::Equals,
                18 => Token::NotEquals,
                19 => Token::LessOrEquals,
                20 => Token::GreaterOrEquals,
                21 => Token::Greater,
                22 => Token::Less,
                23 => Token::Assignment,
                24 => Token::Concat,
                25 => Token::Plus,
                26 => Token::Minus,
                27 => Token::Mul,
                28 => Token::Div,
                29 => Token::Mod,
                30 => Token::Exp,
                31 => Token::OpenRoundBracket,
                32 => Token::CloseRoundBracket,
                33 => Token::OpenSquareBracket,
                34 => Token::CloseSquareBracket,
                35 => Token::OpenSquigglyBracket,
                36 => Token::CloseSquigglyBracket,
                37 => Token::Colon,
                38 => Token::Dot,
                39 => Token::Comma,
                40 => Token::Semicolon,
                41 => Token::Ident(Ident::arbitrary(g)),
                42 => Token::String(StringLiteral::arbitrary(g)),
                43 => Token::Number(NumberLiteral::arbitrary(g)),
                _ => unreachable!(),
            }
        }
    }

    macro_rules! assert_tokens {
        ($name:ident, $text:tt) => {
            #[test]
            fn $name() {
                static TEXT: &str = indoc::indoc!($text);
                let tokens: Vec<Token> = Token::lexer(TEXT).into_iter().collect();
                assert!(
                    tokens.iter().all(|token| !token.is_err()),
                    "Token stream contains Token::Error,\n{:?}",
                    tokens
                );
                insta::assert_debug_snapshot!(tokens);
            }
        };
    }

    assert_tokens!(
        function_1,
        "
            function remove_blanks (s)
                local b = strfind(s, ' ')
                while b do
                    s = strsub(s, 1, b-1) .. strsub(s, b+1)
                    b = strfind(s, ' ')
                end
                return s
            end
        "
    );

    assert_tokens!(
        function_2,
        "
            function f (t)                  -- t is a table
                local i, v = next(t, nil)   -- i is an index of t, v = t[i]
                while i do
                    -- do something with i and v
                    i, v = next(t, i)       -- get next index
                end
            end
        "
    );

    assert_tokens!(string_literal, "\"hello world \\n \\\"nope\\\"\"");

    assert_tokens!(simple_number, "4 1000 10000000000000000 -60 +728");
    assert_tokens!(
        fractional_number,
        "4.23 .23 4. -4.67 -.25 -8. +.24 +5. +4.27, -.1234567890123456789"
    );
    assert_tokens!(exponents, "4e10 .15e-7 5.e+8 -6e7 -5.24e-7 +.8e+1");
}
