use std::fmt::Display;

use super::Token;

pub struct Formatting {
    before: FormattingStyle,
    after: FormattingStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormattingStyle {
    Condensed,
    Space,
    StrictSpace,
    Newline,
    Indent(IndentationChange),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum IndentationChange {
    Increase = 1,
    Decrease = -1,
}

pub fn formatting_style_precedence(
    left: FormattingStyle,
    right: FormattingStyle,
) -> FormattingStyle {
    use FormattingStyle::*;
    match (left, right) {
        (Condensed, Space) => Condensed,
        (Space, Condensed) => Condensed,

        (Condensed, StrictSpace) => StrictSpace,
        (StrictSpace, Condensed) => StrictSpace,

        (Indent(c), _) => Indent(c),
        (_, Indent(c)) => Indent(c),

        (Newline, _) => Newline,
        (_, Newline) => Newline,

        // when a == b
        (a, _) => a,
    }
}

impl Token {
    pub fn formatting(&self) -> Formatting {
        use FormattingStyle::*;
        use IndentationChange::*;
        use Token::*;
        match self {
            Error | And | Nil | Not | Or | Equals | NotEquals | LessOrEquals | GreaterOrEquals
            | Greater | Less | Concat | Plus | Minus | Mul | Div | Mod | OpenSquigglyBracket
            | CloseSquigglyBracket | Ident(_) | String(_) | Number(_) => Formatting {
                before: Space,
                after: Space,
            },
            Function | If | Local | Return | While => Formatting {
                before: Newline,
                after: Space,
            },
            Else | ElseIf => Formatting {
                before: Indent(Decrease),
                after: Indent(Increase),
            },
            End => Formatting {
                before: Indent(Decrease),
                after: Newline,
            },
            Until => Formatting {
                before: Indent(Decrease),
                after: Space
            },
            Then | Do | Repeat => Formatting {
                before: Space,
                after: Indent(Increase),
            },
            Exp | OpenRoundBracket | CloseRoundBracket | OpenSquareBracket | CloseSquareBracket
            | Dot | Colon => Formatting {
                before: Condensed,
                after: Condensed,
            },
            Comma | Semicolon => Formatting {
                before: Condensed,
                after: Space,
            },
            Assignment => Formatting {
                before: StrictSpace,
                after: StrictSpace,
            },
        }
    }
}

pub fn format_single_token(
    token: Token,
    indent: &mut i32,
    current_format: &mut FormattingStyle,
    fmt: &mut std::fmt::Formatter,
) -> std::fmt::Result {
    use FormattingStyle::*;
    let Formatting { before, after } = token.formatting();
    match formatting_style_precedence(*current_format, before) {
        Condensed => {}
        Space | StrictSpace => ' '.fmt(fmt)?,
        Newline => {
            '\n'.fmt(fmt)?;
            for _ in 0..*indent {
                '\t'.fmt(fmt)?;
            }
        }
        Indent(change) => {
            '\n'.fmt(fmt)?;
            *indent += change as i32;
            for _ in 0..*indent {
                '\t'.fmt(fmt)?;
            }
        }
    };
    *current_format = after;
    token.fmt(fmt)?;
    Ok(())
}

pub fn format_tokens(
    tokens: &mut impl Iterator<Item = Token>,
    fmt: &mut std::fmt::Formatter,
) -> std::fmt::Result {
    let mut indent = 0;
    let mut current_format = FormattingStyle::Condensed;
    for token in tokens {
        format_single_token(token, &mut indent, &mut current_format, fmt)?;
    }
    Ok(())
}

#[macro_export]
macro_rules! fmt_tokens {
    ($type:ty) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                crate::lex::format::format_tokens(&mut self.clone().to_tokens(), fmt)
            }
        }
    };
}
