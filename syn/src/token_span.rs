#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenSpan {
    Unknown,
    StreamPosition(usize),
    SourceByteSpan { start: usize, end: usize },
}

impl From<std::ops::Range<usize>> for TokenSpan {
    fn from(range: std::ops::Range<usize>) -> Self {
        Self::SourceByteSpan {
            start: range.start,
            end: range.end,
        }
    }
}

impl std::fmt::Display for TokenSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StreamPosition(pos) => write!(f, "StreamPosition({})", pos),
            Self::SourceByteSpan { start, end } => write!(f, "{}..{}", start, end),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    col: usize,
    row: usize,
}

// Source always in utf-8
pub fn find_source_position(source: &str, byte_offset: usize) -> Option<SourcePosition> {
    let mut row = 0;
    let mut col = 0;
    let mut gone_through = 0;
    for char in source.chars() {
        if gone_through >= byte_offset {
            return Some(SourcePosition { col, row });
        }
        col += 1;
        if char == '\n' {
            row += 1;
            col = 0;
        }
        gone_through += char.len_utf8();
    }
    None
}

#[cfg(test)]
mod test {
    use indoc::indoc;

    use crate::{find_source_position, SourcePosition};

    #[test]
    fn span_traversal_correctly_identifies_position() {
        let source = indoc! {"
            function fib(n)
                if n == 0 then
                    return 0
                elseif n == 1 then
                    return 1
                else
                    return fib(fib(n - 1) + fib(n - 2))
                end
            end
        "};

        let expectations = [
            (0, SourcePosition { col: 0, row: 0 }),
            (9, SourcePosition { col: 9, row: 0 }),
            (16, SourcePosition { col: 0, row: 1 }),
            (50, SourcePosition { col: 15, row: 2 }),
            (156, SourcePosition { col: 3, row: 8 }),
        ];

        let not_in_source = [157, 200, 20_000];

        for (byte_offset, source_position) in expectations {
            assert_eq!(
                find_source_position(source, byte_offset),
                Some(source_position)
            );
        }

        for byte_offset in not_in_source {
            assert_eq!(find_source_position(source, byte_offset), None);
        }
    }
}
