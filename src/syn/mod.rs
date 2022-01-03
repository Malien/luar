use crate::lex::{Ident, Token};
use crate::util::NonEmptyVec;

pub mod expr;
pub use expr::op::*;
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

pub type ParseError = peg::error::ParseError<usize>;

#[cfg(test)]
pub mod string_parser {
    macro_rules! forward {
        ($rule: ident, $ret: ty) => {
            pub fn $rule(input: &str) -> Result<$ret, crate::syn::ParseError> {
                use logos::Logos;
                let tokens: Vec<_> = crate::lex::Token::lexer(input).collect();
                crate::syn::lua_parser::$rule(&tokens)
            }
        };
    }

    forward!(expression, super::Expression);
    forward!(module, super::Module);
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

        pub rule var_or_func_expression() -> Expression
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
            = lfield:lfieldlist()? ffield:ffieldlist()? {
                let mut lfield = lfield.unwrap_or_default();
                lfield.reverse();
                let mut ffield = ffield.unwrap_or_default();
                ffield.reverse();
                TableConstructor { lfield, ffield }
            }

        rule lfieldlist() -> Vec<Expression>
            = head:expression() !(_:[Token::Assignment]) tail:_lfieldlist_after_expr() {
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
                let mut tail = tail;
                tail.push(head);
                tail.reverse();
                // SAFETY: I've pushed head into vec
                unsafe { NonEmptyVec::new_unchecked(tail) }
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
                let mut tail = tail;
                tail.push(head);
                tail.reverse();
                // SAFETY: I've pushed head into vec
                unsafe { NonEmptyVec::new_unchecked(tail) }
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
            = _:[Token::Return] expr:expression()? { Return(expr) }

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
                let tokens: Vec<_> = crate::lex::Token::lexer($input).collect();
                let parsed = crate::syn::lua_parser::$type(&tokens).unwrap();
                assert_eq!(parsed, $expected)
            }
        };
    }

    #[macro_export]
    macro_rules! assert_parses {
        ($type: ident, $expected: expr) => {{
            let expected = $expected;
            let tokens: Vec<_> = crate::lex::ToTokenStream::to_tokens(expected.clone()).collect();
            let parsed = crate::syn::lua_parser::$type(&tokens).unwrap();
            assert_eq!(expected, parsed);
        }};
    }

}
