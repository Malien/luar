use std::collections::HashMap;

use keyed_vec::KeyedVec;
use luar_lex::Ident;

use crate::{
    lang::LuaValue,
    opt::syn::{Chunk, FunctionDeclaration, FunctionName},
};

use super::syn::{
    Assignment, Block, Conditional, ConditionalTail, Declaration, Expression, FunctionCall,
    FunctionCallArgs, GlobalValueID, LocalValueID, Module, RepeatLoop, Return, Statement,
    TableConstructor, ValueID, Var, WhileLoop,
};

#[derive(Debug, Clone, Default)]
pub struct GlobalValues {
    pub cells: KeyedVec<GlobalValueID, LuaValue>,
    pub mapping: HashMap<String, GlobalValueID>,
}

pub struct LocalValues<'a> {
    pub current: LocalValueID,
    pub mapping: HashMap<String, LocalValueID>,
    pub globals: &'a mut GlobalValues,
}

impl GlobalValues {
    pub fn id_for(&mut self, ident: impl AsRef<str>) -> GlobalValueID {
        self.mapping
            .entry(ident.as_ref().to_owned())
            .or_insert_with(|| self.cells.push(LuaValue::Nil))
            .clone()
    }
}

impl<'a> LocalValues<'a> {
    fn id_for(&mut self, ident: impl AsRef<str>) -> ValueID {
        if let Some(&id) = self.mapping.get(ident.as_ref()) {
            return ValueID::Local(id);
        }
        return ValueID::Global(self.globals.id_for(ident));
    }

    fn delcare(&mut self, ident: Ident) -> LocalValueID {
        let id = self.current;
        self.current.0 += 1;
        self.mapping.insert(ident.into(), id);
        id
    }

    fn new(globals: &'a mut GlobalValues) -> Self {
        Self {
            current: LocalValueID(0),
            mapping: HashMap::new(),
            globals,
        }
    }

    fn globals(&mut self) -> &mut GlobalValues {
        self.globals
    }
}

pub fn compile_module(module: luar_syn::Module) -> Module {
    let mut global_state = GlobalValues::default();
    let mut root_locals = LocalValues::new(&mut global_state);

    let chunks = module
        .chunks
        .into_iter()
        .map(|chunk| match chunk {
            luar_syn::Chunk::FnDecl(decl) => Chunk::FnDecl(compile_fn_body(&mut root_locals, decl)),
            luar_syn::Chunk::Statement(statement) => {
                Chunk::Statement(compile_statement(&mut root_locals, statement))
            }
        })
        .collect();
    let ret = module.ret.map(|ret| compile_ret(&mut root_locals, ret));

    Module { chunks, ret }
}

fn compile_fn_body(
    root_locals: &mut LocalValues,
    decl: luar_syn::FunctionDeclaration,
) -> FunctionDeclaration {
    let name = match decl.name {
        luar_syn::FunctionName::Plain(var) => FunctionName::Plain(compile_var(root_locals, var)),
        luar_syn::FunctionName::Method(var, ident) => {
            FunctionName::Method(compile_var(root_locals, var), ident.clone())
        }
    };
    let globals = root_locals.globals();
    let mut fn_locals = LocalValues::new(globals);
    let args = decl
        .args
        .iter()
        .map(|ident| fn_locals.delcare(ident.clone()))
        .collect();
    let body = compile_block(&mut fn_locals, decl.body);
    FunctionDeclaration { name, args, body }
}

fn compile_statement(locals: &mut LocalValues, statement: luar_syn::Statement) -> Statement {
    match statement {
        luar_syn::Statement::Assignment(assignment) => Statement::Assignment(Assignment {
            names: assignment.names.map(|var| compile_var(locals, var)),
            values: assignment.values.map(|expr| compile_expr(locals, expr)),
        }),
        luar_syn::Statement::LocalDeclaration(decl) => {
            let initial_values = decl
                .initial_values
                .into_iter()
                .map(|expr| compile_expr(locals, expr))
                .collect();
            let names = decl.names.map_ref(|ident| locals.delcare(ident.clone()));
            Statement::LocalDeclaration(Declaration {
                names,
                initial_values,
            })
        }
        luar_syn::Statement::While(while_loop) => Statement::While(WhileLoop {
            condition: compile_expr(locals, while_loop.condition),
            body: compile_block(locals, while_loop.body),
        }),
        luar_syn::Statement::Repeat(repeat_loop) => Statement::Repeat(RepeatLoop {
            body: compile_block(locals, repeat_loop.body),
            condition: compile_expr(locals, repeat_loop.condition),
        }),
        luar_syn::Statement::If(conditional) => Statement::If(compile_if(locals, conditional)),
        luar_syn::Statement::FunctionCall(fn_call) => {
            Statement::FunctionCall(compile_fn_call(locals, fn_call))
        }
    }
}

fn compile_var(locals: &mut LocalValues, var: luar_syn::Var) -> Var {
    match var {
        luar_syn::Var::Named(ident) => Var::Named(locals.id_for(ident)),
        luar_syn::Var::PropertyAccess { from, property } => Var::PropertyAccess {
            from: Box::new(compile_var(locals, *from)),
            property,
        },
        luar_syn::Var::MemberLookup { from, value } => Var::MemberLookup {
            from: Box::new(compile_var(locals, *from)),
            value: Box::new(compile_expr(locals, *value)),
        },
    }
}

fn compile_expr(locals: &mut LocalValues, expr: luar_syn::Expression) -> Expression {
    match expr {
        luar_syn::Expression::Nil => Expression::Nil,
        luar_syn::Expression::String(str) => Expression::String(str),
        luar_syn::Expression::Number(num) => Expression::Number(num),
        luar_syn::Expression::Variable(var) => Expression::Variable(compile_var(locals, var)),
        luar_syn::Expression::BinaryOperator { lhs, op, rhs } => Expression::BinaryOperator {
            lhs: Box::new(compile_expr(locals, *lhs)),
            op,
            rhs: Box::new(compile_expr(locals, *rhs)),
        },
        luar_syn::Expression::UnaryOperator { op, exp } => Expression::UnaryOperator {
            op,
            exp: Box::new(compile_expr(locals, *exp)),
        },
        luar_syn::Expression::TableConstructor(tbl) => {
            Expression::TableConstructor(compile_table_constructor(locals, tbl))
        }
        luar_syn::Expression::FunctionCall(fn_call) => {
            Expression::FunctionCall(compile_fn_call(locals, fn_call))
        }
    }
}

fn compile_block(locals: &mut LocalValues, block: luar_syn::Block) -> Block {
    Block {
        statements: block
            .statements
            .into_iter()
            .map(|stmt| compile_statement(locals, stmt))
            .collect(),
        ret: block.ret.map(|ret| compile_ret(locals, ret)),
    }
}

fn compile_ret(locals: &mut LocalValues, ret: luar_syn::Return) -> Return {
    Return(
        ret.0
            .into_iter()
            .map(|expr| compile_expr(locals, expr))
            .collect(),
    )
}

fn compile_if(locals: &mut LocalValues, conditional: luar_syn::Conditional) -> Conditional {
    Conditional {
        condition: compile_expr(locals, conditional.condition),
        body: compile_block(locals, conditional.body),
        tail: compile_if_tail(locals, conditional.tail),
    }
}

fn compile_if_tail(locals: &mut LocalValues, tail: luar_syn::ConditionalTail) -> ConditionalTail {
    match tail {
        luar_syn::ConditionalTail::End => ConditionalTail::End,
        luar_syn::ConditionalTail::Else(block) => {
            ConditionalTail::Else(compile_block(locals, block))
        }
        luar_syn::ConditionalTail::ElseIf(conditional) => {
            ConditionalTail::ElseIf(Box::new(compile_if(locals, *conditional)))
        }
    }
}

fn compile_fn_call(locals: &mut LocalValues, fn_call: luar_syn::FunctionCall) -> FunctionCall {
    match fn_call {
        luar_syn::FunctionCall::Method { func, method, args } => FunctionCall::Method {
            func: compile_var(locals, func),
            method,
            args: compile_fn_call_args(locals, args),
        },
        luar_syn::FunctionCall::Function { func, args } => FunctionCall::Function {
            func: compile_var(locals, func),
            args: compile_fn_call_args(locals, args),
        },
    }
}

fn compile_table_constructor(
    locals: &mut LocalValues,
    tbl: luar_syn::TableConstructor,
) -> TableConstructor {
    TableConstructor {
        lfield: tbl
            .lfield
            .into_iter()
            .map(|expr| compile_expr(locals, expr))
            .collect(),
        ffield: tbl
            .ffield
            .into_iter()
            .map(|(ident, expr)| (ident, compile_expr(locals, expr)))
            .collect(),
    }
}

fn compile_fn_call_args(
    locals: &mut LocalValues,
    args: luar_syn::FunctionCallArgs,
) -> FunctionCallArgs {
    match args {
        luar_syn::FunctionCallArgs::Table(tbl) => {
            FunctionCallArgs::Table(compile_table_constructor(locals, tbl))
        }
        luar_syn::FunctionCallArgs::Arglist(args) => FunctionCallArgs::Arglist(
            args.into_iter()
                .map(|expr| compile_expr(locals, expr))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn removes_named_local_var_lookup() {
        let program = "
            local a = 1
            local b = 2
            local c = a + b
            return c
        ";
        let ast = luar_syn::lua_parser::module(program).unwrap();
        let module = super::compile_module(ast);

        insta::assert_debug_snapshot!(module);
    }

    #[test]
    fn removes_named_var_lookup() {
        let program = "
            global = 1
            local l0
            local l1, l2 = 2, { global, l0; bar = l0 }
            function l2:foo(a, b, c)
                if a then
                    return global + l1
                elseif b then
                    return l1 - a
                else
                    return c + 1
                end
            end
            print('hello', global, l0 - l1.bar)
            return global, l1, l2, l2:foo(3, l1, l2[global]), unspecified
        ";
        let ast = luar_syn::lua_parser::module(program).unwrap();
        let module = super::compile_module(ast);

        insta::assert_debug_snapshot!(module);
    }
}
