use logos::{Lexer, Logos};
use thiserror::Error;

/// Syntax
///
/// ```
/// module -> { statement | function }
/// block -> { stat sc } [ret sc]
/// sc -> ';'
/// stat -> varlist1 '=' explist1
/// varlist1 -> var { ',' var }
/// var -> name | var '[' exp1 ']' | var '.' name
/// stat -> 'while' exp1 'do' block 'end'
///       | 'repeat' block 'until' exp1
///       | 'if' exp1 'then' block { elseif } ['else' block] 'end'
///       | functioncall
///       | tableconstructor
///       | 'local' declist [init]
/// elseif -> 'elseif' exp1 'then' block
/// ret -> 'return' explist
/// declist -> name { , 'name' }
/// init -> '=' explist1
/// exp -> '(' exp ')'
///      | 'nil'
///      | number
///      | literal
///      | var
/// tableconstructor -> '@' '(' [exp1] ')' | '@' [name] fieldlist
/// fieldlist -> '{' [ffieldlist1] '}' | '[' [lfieldlist1] ']'
/// ffieldlist1 -> ffield { ',' ffield }
/// ffield -> name '=' exp
/// lfieldlist1 -> exp { ',' exp }
/// functioncall -> var '(' [explist1] ')'
/// explist1 -> { exp1 ',' } exp
/// function -> 'function' name '(' [parlist1] ')' block 'end'
/// parlist1 -> 'name' { ',' name }
/// ```

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

// #[derive(Copy, Clone, Debug, Eq, PartialEq, Logos)]
// enum Token {
//
// }

#[derive(Clone, Debug, PartialEq, Eq, Logos)]
pub enum Token {
    #[error]
    #[regex(r"[ \t\n\f]", logos::skip)]
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
    #[regex(r"[_a-zA-Z][_a-zA-Z0-9]*", make_owned_string)]
    Ident(String),
    #[regex(
        "(\"(?:[^\"'\\\\]|\\\\.)*\")|('(?:[^\"'\\\\]|\\\\.)*')",
        parse_string_literal
    )]
    StringLiteral(String),
    #[regex(
        r"[+-]?((\d+\.\d+)|(\.\d+)|(\d+\.?))(e[+-]?\d+)?",
        parse_number_literal
    )]
    Number(NumberLiteral),
}

#[derive(Error, Debug)]
#[error("Token passed in is not a valid string literal")]
struct StringLiteralParseError;

// enum NumberLiteralParseError {
//     Integral()
// }
type NumberLiteralParseError = num::rational::ParseRatioError;

// Can be optimized not to allocate extra space if not necessary
// enum LuaNumberLiteral {
//     Integral(BigInt),
//     Rational(BigRational)
// }
type NumberLiteral = num::rational::BigRational;

fn parse_string_literal(input: &mut Lexer<Token>) -> Result<String, StringLiteralParseError> {
    let slice = input.slice();
    if slice.len() < 2 {
        return Err(StringLiteralParseError);
    }
    let raw_string: &str = &slice[1..slice.len() - 1];
    // Not a very efficient method to escape characters
    Ok(raw_string
        .replace(r"\\", "\\")
        .replace(r"\r", "\r")
        .replace(r"\n", "\n")
        .replace(r"\t", "\t")
        .replace("\\\"", "\"")
        .replace(r"\'", "'"))
}

// static integer_regex: Regex = Regex::new(r"[+-]?(\d+\.?)(e[+-]?\d+)?").unwrap();

fn parse_number_literal(
    input: &mut Lexer<Token>,
) -> Result<NumberLiteral, NumberLiteralParseError> {
    let slice: &str = input.slice();
    slice.parse()
}

fn make_owned_string(input: &mut Lexer<Token>) -> String {
    input.slice().to_string()
}

#[cfg(test)]
mod tests {
    use super::Token;
    use logos::Logos;

    macro_rules! assert_tokens {
        ($name:ident, $text:tt) => {
            #[test]
            #[function_name::named]
            fn $name() {
                static TEXT: &str = indoc::indoc!($text);
                let tokens: Vec<Token> = Token::lexer(TEXT).into_iter().collect();
                insta::assert_debug_snapshot!(function_name!(), tokens);
            }
        };
    }

    assert_tokens!(function_1, "
        function remove_blanks (s)
            local b = strfind(s, ' ')
            while b do
                s = strsub(s, 1, b-1) .. strsub(s, b+1)
                b = strfind(s, ' ')
            end
            return s
        end
    ");
}
