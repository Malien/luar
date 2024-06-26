#[cfg(test)]
#[cfg(feature = "quickcheck")]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use luar_lex::{Ident, Token};

mod token_stream;
use peg::error::ExpectedSet;
pub use token_stream::*;

mod token_span;
pub use token_span::*;

use non_empty::NonEmptyVec;

pub(crate) mod flat_intersperse;

pub mod expr;
pub use expr::*;

pub mod stmnt;
pub use stmnt::*;

pub mod block;
pub use block::*;

pub mod function_declaration;
pub use function_declaration::*;

pub mod ret;
pub use ret::*;

pub mod module;
pub use module::*;

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

enum FunctionCallHead {
    Function(Var),
    Method(Var, Ident),
}

fn compose_function_call(head: FunctionCallHead, args: FunctionCallArgs) -> FunctionCall {
    match head {
        FunctionCallHead::Function(func) => FunctionCall::Function { func, args },
        FunctionCallHead::Method(func, method) => FunctionCall::Method { func, method, args },
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("{0}")]
    Raw(#[from] RawParseError),
    #[error("{0}")]
    Identified(#[from] ParseErrorWithSourcePosition),
}

pub type RawParseError = peg::error::ParseError<TokenSpan>;

#[derive(Debug, thiserror::Error)]
#[error("Encountered a parsing error at {start:?}")]
pub struct ParseErrorWithSourcePosition {
    expected: ExpectedSet,
    start: Option<SourcePosition>,
    end: Option<SourcePosition>,
}

pub(crate) fn enrich_error(source: &str, raw_error: RawParseError) -> ParseErrorWithSourcePosition {
    let RawParseError { expected, location } = raw_error;
    match location {
        TokenSpan::SourceByteSpan { start, end } => ParseErrorWithSourcePosition {
            expected,
            start: find_source_position(source, start),
            end: find_source_position(source, end),
        },
        TokenSpan::Unknown => ParseErrorWithSourcePosition {
            expected,
            start: None,
            end: None,
        },
        TokenSpan::StreamPosition(_) => ParseErrorWithSourcePosition {
            expected,
            start: None,
            end: None,
        },
    }
}

pub mod lua_parser {
    macro_rules! forward {
        ($rule: ident, $ret: ty) => {
            pub fn $rule(input: &str) -> Result<$ret, crate::ParseErrorWithSourcePosition> {
                use logos::Logos;
                let tokens: crate::TokenStream = luar_lex::Token::lexer(input).spanned().collect();
                crate::lua_token_parser::$rule(&tokens)
                    .map_err(|error| super::enrich_error(input, error))
            }
        };
    }

    forward!(nil, super::Expression);
    forward!(string, super::Expression);
    forward!(number, super::Expression);
    forward!(var_expression, super::Expression);
    forward!(tbl_expression, super::Expression);
    forward!(expression, super::Expression);
    forward!(block, super::Block);
    forward!(function_call, super::FunctionCall);
    forward!(table_constructor, super::TableConstructor);
    forward!(var, super::Var);
    forward!(function_declaration, super::FunctionDeclaration);
    forward!(ret, super::Return);
    forward!(declaration, super::Declaration);
    forward!(assignment, super::Assignment);
    forward!(conditional, super::Conditional);
    forward!(statement, super::Statement);
    forward!(repeat_loop, super::RepeatLoop);
    forward!(while_loop, super::WhileLoop);
    forward!(module, super::Module);
}

pub mod unspanned_lua_token_parser {
    macro_rules! forward {
        ($rule: ident, $ret: ty) => {
            pub fn $rule(
                input: impl ::std::iter::IntoIterator<Item = luar_lex::Token>,
            ) -> Result<$ret, $crate::RawParseError> {
                $crate::lua_token_parser::$rule(&$crate::ToTokenStreamExt::to_spanned_token_stream(
                    input.into_iter(),
                ))
            }
        };
    }

    forward!(nil, super::Expression);
    forward!(string, super::Expression);
    forward!(number, super::Expression);
    forward!(var_expression, super::Expression);
    forward!(tbl_expression, super::Expression);
    forward!(expression, super::Expression);
    forward!(block, super::Block);
    forward!(function_call, super::FunctionCall);
    forward!(table_constructor, super::TableConstructor);
    forward!(var, super::Var);
    forward!(function_declaration, super::FunctionDeclaration);
    forward!(ret, super::Return);
    forward!(declaration, super::Declaration);
    forward!(assignment, super::Assignment);
    forward!(conditional, super::Conditional);
    forward!(statement, super::Statement);
    forward!(repeat_loop, super::RepeatLoop);
    forward!(while_loop, super::WhileLoop);
    forward!(module, super::Module);
}

peg::parser! {
    pub grammar lua_token_parser() for TokenStream {
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

        rule var_or_func_expression() -> Expression
            = func:var() _:[Token::Colon] _:[Token::Ident(method)] args:function_call_args() {
                Expression::FunctionCall(FunctionCall::Method { func, method, args })
            }
            / func:var() args:function_call_args() {
                Expression::FunctionCall(FunctionCall::Function { func, args })
            }
            / var:var()  { Expression::Variable(var) }

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
            e:tbl_expression() { e }
            e:var_or_func_expression() { e }
            _: [Token::OpenRoundBracket] e:expression() _:[Token::CloseRoundBracket] { e }
        }

        pub rule ident() -> Ident
            = _:[Token::Ident(ident)] { ident }

        pub rule named() -> Var
            = ident:ident() { Var::Named(ident) }

        pub rule property_access() -> Var
            = base:var() _:[Token::Dot] property:ident() {
                Var::PropertyAccess {
                    from: Box::new(base),
                    property
                }
            }

        pub rule var() -> Var
            = base:named() leftovers:_var() { accumulate_var_leftovers(base, leftovers) }

        rule _var() -> VarLeftover
            = _:[Token::Dot] ident:ident() next:_var() {
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
            = ffield:ffieldlist() {
                let mut ffield = ffield;
                ffield.reverse();
                TableConstructor { ffield, lfield: vec![] }
            }
            / lfield:lfieldlist() ffield:ffield_tail()? {
                let mut lfield = lfield;
                lfield.reverse();
                let mut ffield = ffield.unwrap_or_default();
                ffield.reverse();
                TableConstructor { lfield, ffield }
            }

        rule ffield_tail() -> Vec<(Ident, Expression)> 
            = _:[Token::Semicolon] ffield:ffieldlist()? {
                ffield.unwrap_or_default()
            }

        rule lfieldlist() -> Vec<Expression>
            = head:expression() !(_:[Token::Assignment]) tail:_lfieldlist_after_expr() {
                let mut tail = tail;
                tail.push(head);
                tail
            }
            / { Vec::new() }

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

        rule ffieldlist() -> Vec<(Ident, Expression)>
            = _:[Token::Semicolon]? head:name_pair() tail:_ffieldlist_after_pair() {
                let mut tail = tail;
                tail.push(head);
                tail
            }

        rule name_pair() -> (Ident, Expression)
            = ident:ident() _:[Token::Assignment] expr:expression() { (ident, expr) }

        rule _ffieldlist_after_pair() -> Vec<(Ident, Expression)>
            = _: [Token::Comma] rest:_ffieldlist() { rest }
            / { Vec::new() }

        rule _ffieldlist() -> Vec<(Ident, Expression)>
            = head:name_pair() tail:_ffieldlist_after_pair() {
                let mut tail = tail;
                tail.push(head);
                tail
            }
            / { Vec::new() }

        pub rule function_call() -> FunctionCall
            = head:function_call_head() args:function_call_args() {
                compose_function_call(head, args)
            }

        rule function_call_head() -> FunctionCallHead
            = func:var() _:[Token::Colon] ident:ident() { FunctionCallHead::Method(func, ident) }
            / func:var() { FunctionCallHead::Function(func) }

        rule function_call_args() -> FunctionCallArgs
            = _:[Token::OpenRoundBracket] exprs:exprlist() _:[Token::CloseRoundBracket] {
                FunctionCallArgs::Arglist(exprs)
            }
            / tbl:table_constructor() {
                FunctionCallArgs::Table(tbl)
            }

        rule exprlist1() -> NonEmptyVec<Expression>
            = head:expression() tail:_exprlist() {
                let mut exprlist = NonEmptyVec::new_with_tail(tail, head);
                exprlist.reverse();
                exprlist
            }

        rule exprlist() -> Vec<Expression>
            = exprs:exprlist1() { exprs.into() }
            / { Vec::new() }

        rule _exprlist() -> Vec<Expression>
            = _:[Token::Comma] head:expression() tail:_exprlist() {
                let mut tail = tail;
                tail.push(head);
                tail
            }
            / { Vec::new() }

        pub rule statement() -> Statement
            = s:_statement() _:[Token::Semicolon]? { s }

        rule _statement() -> Statement
            = assignment:assignment() { Statement::Assignment(assignment) }
            / decl:declaration() { Statement::LocalDeclaration(decl) }
            / while_loop:while_loop() { Statement::While(while_loop) }
            / repeat_loop:repeat_loop() { Statement::Repeat(repeat_loop) }
            / conditional:conditional() { Statement::If(conditional) }
            / function_call:function_call() { Statement::FunctionCall(function_call) }

        pub rule assignment() -> Assignment
            = names:varlist1() _:[Token::Assignment] values:exprlist1() {
                Assignment { names, values }
            }

        rule varlist1() -> NonEmptyVec<Var>
            = head:var() tail:_varlist() {
                let mut varlist = NonEmptyVec::new_with_tail(tail, head);
                varlist.reverse();
                varlist
            }

        rule varlist() -> Vec<Var>
            = vars:varlist1() { vars.into() }
            / { Vec::new() }

        rule _varlist() -> Vec<Var>
            = _:[Token::Comma] head:var() tail:_varlist() {
                let mut tail = tail;
                tail.push(head);
                tail
            }
            / { Vec::new() }

        pub rule declaration() -> Declaration
            = _:[Token::Local] names:decllist1() initial_values:initial_values() {
                Declaration { names, initial_values, }
            }

        rule initial_values() -> Vec<Expression>
            = _:[Token::Assignment] values:exprlist1() { values.into() }
            / { Vec::new() }

        rule decllist1() -> NonEmptyVec<Ident>
            = decls:ident() ++ [Token::Comma] {
                // SAFETY: It is guaranteed by peg, that decls contains at least one element
                unsafe { NonEmptyVec::new_unchecked(decls) }
            }

        pub rule while_loop() -> WhileLoop
            = _:[Token::While] condition:expression() _:[Token::Do] body:block() _:[Token::End] {
                WhileLoop { condition, body }
            }

        pub rule block() -> Block
            = statements:statement()* ret:ret()? { Block { statements, ret: ret } }

        pub rule repeat_loop() -> RepeatLoop
            = _:[Token::Repeat] body:block() _:[Token::Until] condition:expression() {
                RepeatLoop { body, condition }
            }

        pub rule conditional() -> Conditional
            = _:[Token::If] condition:expression() _:[Token::Then] body:block() tail:conditional_tail() {
                Conditional { condition, body, tail }
            }

        rule conditional_tail() -> ConditionalTail
            = _:[Token::End] { ConditionalTail::End }
            / _:[Token::Else] body:block() _:[Token::End] { ConditionalTail::Else(body) }
            / _:[Token::ElseIf] condition:expression() _:[Token::Then] body:block() tail:conditional_tail() {
                ConditionalTail::ElseIf(Box::new(Conditional { condition, body, tail }))
            }

        pub rule ret() -> Return
            = _:[Token::Return] exprs:expression() ** [Token::Comma] {
                Return(exprs)
            }

        pub rule function_declaration() -> FunctionDeclaration
            = _:[Token::Function] name:function_name() args:function_args_decl() body:block() _:[Token::End] {
                FunctionDeclaration {
                    name,
                    args,
                    body,
                }
            }

        rule function_name() -> FunctionName
            = name:var() _:[Token::Colon] method:ident() { FunctionName::Method(name, method) }
            / name:var() { FunctionName::Plain(name) }

        rule function_args_decl() -> Vec<Ident>
            = _:[Token::OpenRoundBracket] args:ident() ** [Token::Comma] _:[Token::CloseRoundBracket] {
                args
            }

        pub rule module() -> Module
            = chunks:chunk()* ret:ret()? {
                Module { chunks, ret }
            }

        rule chunk() -> Chunk
            = statement:statement() { Chunk::Statement(statement) }
            / decl:function_declaration() { Chunk::FnDecl(decl) }
    }
}

#[cfg(test)]
mod tests {
    #[macro_export]
    macro_rules! input_parsing_expectation {
        ($type: ident, $name: tt, $input: expr, $expected: expr) => {
            #[test]
            fn $name() {
                use logos::Logos;
                let tokens: crate::TokenStream = luar_lex::Token::lexer($input).spanned().collect();
                let parsed = crate::lua_token_parser::$type(&tokens).unwrap();
                assert_eq!(parsed, $expected)
            }
        };
    }

    #[macro_export]
    macro_rules! assert_parses {
        ($type: ident, $expected: expr) => {{
            let expected = $expected;
            let parsed = crate::unspanned_lua_token_parser::$type(
                luar_lex::ToTokenStream::to_tokens(expected.clone()),
            )
            .unwrap();
            assert_eq!(expected, parsed);
        }};
    }
}
