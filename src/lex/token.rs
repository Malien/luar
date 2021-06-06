use logos::Logos;

use super::{Ident, NumberLiteral, StringLiteral};

/// Reserved words:
///     `and` `do` `else` `elseif` `end` `function` `if` `local` `nil` `not`
///     `or` `repeat` `return` `until` `then` `while`
///
/// Other tokens:
///     `~=` `<=` `>=` `<` `>` `=` `..` `+` `-` `*` `/` `%` `(` `)` `{` `}` `[` `]` `@` `;` `,` `.`
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
    #[token("~=")]
    NotEquals,
    #[token("<=")]
    LessOrEquals,
    #[token(">=")]
    GreaterOrEquals,
    #[token(">")]
    Greater,
    #[token("<")]
    Lesser,
    #[token("=")]
    Equals,
    #[token("..")]
    Concat,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiply,
    #[token("/")]
    Div,
    #[token("%")]
    Mod,
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
    #[token("@")]
    At,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    // SAFETY: This is the same regex used to check for identifier validity
    #[regex(r"[_a-zA-Z][_a-zA-Z0-9]*", |str| unsafe { Ident::from_raw(str.slice()) })]
    Ident(Ident),
    #[regex(
        "(\"(?:[^\"'\\\\]|\\\\.)*\")|('(?:[^\"'\\\\]|\\\\.)*')",
        |token| token.slice().parse()
    )]
    String(StringLiteral),
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

    use super::Token;
    use logos::Logos;
    use quickcheck::Arbitrary;

    impl Arbitrary for Token {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let idx = u8::arbitrary(g) % 42;
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
                17 => Token::NotEquals,
                18 => Token::LessOrEquals,
                19 => Token::GreaterOrEquals,
                20 => Token::Greater,
                21 => Token::Lesser,
                22 => Token::Equals,
                23 => Token::Concat,
                24 => Token::Plus,
                25 => Token::Minus,
                26 => Token::Multiply,
                27 => Token::Div,
                28 => Token::Mod,
                29 => Token::OpenRoundBracket,
                30 => Token::CloseRoundBracket,
                31 => Token::OpenSquareBracket,
                32 => Token::CloseSquareBracket,
                33 => Token::OpenSquigglyBracket,
                34 => Token::CloseSquigglyBracket,
                35 => Token::At,
                36 => Token::Dot,
                37 => Token::Comma,
                38 => Token::Semicolon,
                39 => {
                    // Ident(String),
                    todo!()
                }
                40 => {
                    // String(StringLiteral)
                    todo!()
                }
                41 => {
                    // Number(NumberLiteral)
                    todo!()
                }
                _ => unreachable!()
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
