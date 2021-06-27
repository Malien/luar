use std::{
    iter::{Cloned, Enumerate},
    num::NonZeroUsize,
};

use crate::lex::{NumberLiteral, StringLiteral, Token};
use nom::{
    alt, map, named, tag, verify, Err, IResult, InputIter, InputLength, InputTake, Needed, Parser,
};
use thiserror::Error;

/// Syntax
///
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
///      | exp operator exp
///      | unaryoperator exp
///      | tableconstructor
///      | functioncall
/// tableconstructor -> '@' '(' [exp1] ')' | '@' [name] fieldlist
/// fieldlist -> '{' [ffieldlist1] '}' | '[' [lfieldlist1] ']'
/// ffieldlist1 -> ffield { ',' ffield }
/// ffield -> name '=' exp
/// lfieldlist1 -> exp { ',' exp }
/// functioncall -> var '(' [explist1] ')'
/// explist1 -> { exp1 ',' } exp
/// function -> 'function' name '(' [parlist1] ')' block 'end'
/// parlist1 -> 'name' { ',' name }

#[derive(Error, Debug)]
enum ParsingError {
    #[error("Invalid token found (expected {expected:?}, found {found:?})")]
    ExpectationFailure { expected: Token, found: Token },
    #[error("Unexpected EOF found (expected {expected:?})")]
    EOF { expected: Token },
}

fn expectation_failure<T>(expected: Token, found: Token) -> ParsingResult<'static, T> {
    Err(Err::Error(ParsingError::ExpectationFailure {
        expected,
        found,
    }))
}

type ParsingResult<'a, T> = IResult<&'a [Token], T, ParsingError>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Operator {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum TableConstructor {}

#[derive(Debug, PartialEq, Clone)]
enum Expression {
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
enum Var {
    Named(String),
    PropertyAccess {
        from: Box<Var>,
        value: Box<Expression>,
    },
    MemberLookup {
        from: Box<Var>,
        property: String,
    },
}

struct TokenParser {
    token: Token,
}

impl<'a> Parser<&'a [Token], Token, ParsingError> for TokenParser {
    fn parse(&mut self, input: &'a [Token]) -> IResult<&'a [Token], Token, ParsingError> {
        let res = take_one(input).and_then(|(rest, token)| {
            if token == self.token {
                Ok((rest, token))
            } else {
                Err(Err::Error(ParsingError::ExpectationFailure {
                    expected: self.token.clone(),
                    found: token,
                }))
            }
        });
        res
    }
}

fn take_one(input: &[Token]) -> ParsingResult<Token> {
    if input.len() > 0 {
        let token = input[0].clone();
        let rest = &input[1..];
        Ok((rest, token))
    } else {
        Err(Err::Incomplete(Needed::new(1)))
    }
}

fn parse_expr(input: &[Token]) -> ParsingResult<Expression> {
    let (rest, token) = take_one(input)?;
    match token {
        Token::Nil => Ok((rest, Expression::Nil)),
        Token::Number(literal) => Ok((rest, Expression::Number(literal))),
        Token::String(literal) => Ok((rest, Expression::String(literal))),
        _ => expectation_failure(Token::Nil, token), // TODO: expected expression, not Nil
    }
}

pub struct MySlice<'a, T>(pub &'a [T]);

impl<'a, T> InputTake for MySlice<'a, T> {
    fn take(&self, count: usize) -> Self {
        MySlice(&self.0[0..count])
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (prefix, suffix) = self.0.split_at(count);
        (MySlice(suffix), MySlice(prefix))
    }
}

impl<'a, T> InputLength for MySlice<'a, T> {
    fn input_len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, T> InputIter for MySlice<'a, T> {
    type Item = &'a T;

    type Iter = Enumerate<<&'a [T] as IntoIterator>::IntoIter>;

    type IterElem = <&'a [T] as IntoIterator>::IntoIter;

    fn iter_indices(&self) -> Self::Iter {
        self.0.iter().enumerate()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.0.iter()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.0
            .iter()
            .enumerate()
            .find(|(idx, elem)| predicate(elem))
            .map(|(idx, _)| idx)
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        
    }
}

named!(parse_nil<MySlice<'_, Token>, Expression>, tag!(Token::Nil));

#[cfg(test)]
mod tests {
    use super::{parse_expr, Expression, ParsingError};
    use crate::lex::{NumberLiteral, StringLiteral, Token};

    type ReturnType = Result<(), nom::Err<ParsingError>>;

    #[test]
    fn nill_expr() -> ReturnType {
        assert_eq!(Expression::Nil, parse_expr(&[Token::Nil])?.1);

        Ok(())
    }

    #[quickcheck]
    fn number_expr(literal: NumberLiteral) -> ReturnType {
        let expression = parse_expr(&[Token::Number(literal)])?.1;
        match (&literal, &expression) {
            (NumberLiteral(x), Expression::Number(NumberLiteral(y))) if f64::is_nan(*x) => {
                assert!(f64::is_nan(*y))
            }
            _ => assert_eq!(Expression::Number(literal), expression),
        };

        Ok(())
    }

    #[quickcheck]
    fn string_expr(literal: StringLiteral) -> ReturnType {
        assert_eq!(
            Expression::String(literal.clone()),
            parse_expr(&[Token::String(literal)])?.1
        );

        Ok(())
    }

    // fn
}

// fn token<'a>(token: Token, input: &'a[Token]) -> ParsingResult<'a, Token> {
//     Ok((input[0], &input[1..]))
// }

// fn expression(input: &[Token]) -> ParsingResult<Token> {
//     return alt!(
//         map!()
//     );
// }
