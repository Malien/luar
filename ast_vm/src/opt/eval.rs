use super::syn::{
    self, Assignment, Conditional, ConditionalTail, Declaration, Expression, FunctionCall,
    FunctionCallArgs, GlobalValueID, LocalValueID, Module, Return, Statement, TableConstructor,
    ValueID, Var, WhileLoop,
};
use crate::{
    assign_to_value_member, assign_to_value_property,
    binary_op::{
        binary_number_op, concat, greater_or_equals, greater_than, less_or_equals, less_than,
    },
    fn_call::call_value,
    lang::{Context, InnerFn, LuaFunction, LuaKey, LuaValue, ReturnValue, TableRef, TableValue},
    member_lookup, property_access, tail_values,
    unary_op::unary_op_eval,
    ControlFlow, EvalError,
};
use luar_error::{ArithmeticOperator, TypeError};
use luar_lex::{NumberLiteral, StringLiteral};
use luar_syn::BinaryOperator;

pub(crate) type Result<T> = std::result::Result<T, EvalError>;

pub(crate) struct EvalContext<'a> {
    // Let's reuse the stack allocation between function calls.
    context: &'a mut Context,
    offset: usize,
}

impl EvalContext<'_> {
    fn new(context: &mut Context, frame_size: u16) -> EvalContext {
        let offset = context.stack.len();
        let new_size = offset + frame_size as usize;
        context.stack.resize(new_size, LuaValue::Nil);

        EvalContext { context, offset }
    }

    fn local_assign(&mut self, id: LocalValueID, value: LuaValue) {
        self.context.stack[self.offset + id.0 as usize] = value;
    }

    fn local_lookup(&self, id: LocalValueID) -> &LuaValue {
        &self.context.stack[self.offset + id.0 as usize]
    }

    fn global_assign(&mut self, id: GlobalValueID, value: LuaValue) {
        self.context.globals.cells[id] = value;
    }

    fn global_lookup(&self, id: GlobalValueID) -> &LuaValue {
        &self.context.globals.cells[id]
    }

    fn assign(&mut self, id: ValueID, value: LuaValue) {
        match id {
            ValueID::Local(id) => self.local_assign(id, value),
            ValueID::Global(id) => self.global_assign(id, value),
        }
    }

    fn lookup(&self, id: ValueID) -> &LuaValue {
        match id {
            ValueID::Local(id) => self.local_lookup(id),
            ValueID::Global(id) => self.global_lookup(id),
        }
    }
}

impl Drop for EvalContext<'_> {
    fn drop(&mut self) {
        self.context.stack.truncate(self.offset);
    }
}

pub fn eval_module(module: &Module, context: &mut Context) -> Result<ReturnValue> {
    let ctx = &mut EvalContext::new(context, module.local_count);

    for chunk in &module.chunks {
        match chunk {
            syn::Chunk::FnDecl(decl) => eval_fn_decl(decl, ctx)?,
            syn::Chunk::Statement(stmtn) => {
                if let ControlFlow::Return(ret) = eval_stmnt(stmtn, ctx)? {
                    return Ok(ret);
                }
            }
        };
    }

    match module.ret {
        Some(ref ret) => eval_ret(ret, ctx),
        None => Ok(ReturnValue::NIL),
    }
}

pub fn call_function(
    function: &LuaFunction,
    context: &mut Context,
    args: &[LuaValue],
) -> Result<ReturnValue> {
    let InnerFn {
        local_count,
        arg_count,
        ref body,
    } = *function.0;
    let present_arg_count = std::cmp::min(arg_count as usize, args.len());
    let args = &args[..present_arg_count];

    let mut context = EvalContext::new(context, local_count);
    for (idx, value) in args.iter().enumerate() {
        let id = LocalValueID(idx as u16);
        context.local_assign(id, value.clone());
    }

    match call_block(body, &mut context)? {
        ControlFlow::Return(value) => Ok(value),
        ControlFlow::Continue => Ok(ReturnValue::NIL),
    }
}

fn call_block(block: &syn::Block, ctx: &mut EvalContext) -> Result<ControlFlow> {
    for statement in &block.statements {
        if let ControlFlow::Return(value) = eval_stmnt(statement, ctx)? {
            return Ok(ControlFlow::Return(value));
        }
    }

    block
        .ret
        .as_ref()
        .map(|ret| eval_ret(ret, ctx).map(ControlFlow::Return))
        .unwrap_or(Ok(ControlFlow::Continue))
}

fn eval_fn_decl(decl: &syn::FunctionDeclaration, context: &mut EvalContext) -> Result<()> {
    let func = LuaFunction::new(decl);
    let func = LuaValue::Function(func);

    match &decl.name {
        syn::FunctionName::Plain(var) => assign_to_var(&var, func, context),
        syn::FunctionName::Method(_, _) => {
            todo!("Method function declarations are not supported yet")
        }
    }
}

fn eval_stmnt(stmnt: &Statement, ctx: &mut EvalContext) -> Result<ControlFlow> {
    use Statement::*;
    match stmnt {
        Assignment(assignment) => eval_assignment(assignment, ctx).map(|_| ControlFlow::Continue),
        LocalDeclaration(decl) => eval_decl(decl, ctx).map(|_| ControlFlow::Continue),
        FunctionCall(func_call) => eval_fn_call(func_call, ctx).map(|_| ControlFlow::Continue),
        If(conditional) => eval_conditional(conditional, ctx),
        While(while_loop) => eval_while_loop(while_loop, ctx),
        Repeat(_) => todo!("Evaluation of repeat statements is not implemented yet"),
    }
}

fn eval_decl(decl: &syn::Declaration, ctx: &mut EvalContext<'_>) -> Result<()> {
    let Declaration {
        names,
        initial_values,
    } = decl;

    assignment_values(ctx, initial_values)
        .map(|values| multiple_local_assignment(ctx, names.clone(), values))
}

fn multiple_local_assignment(
    ctx: &mut EvalContext,
    names: impl IntoIterator<Item = LocalValueID>,
    values: impl Iterator<Item = LuaValue>,
) {
    for (name, value) in names.into_iter().zip(values) {
        ctx.local_assign(name, value);
    }
}

fn eval_while_loop(while_loop: &syn::WhileLoop, ctx: &mut EvalContext) -> Result<ControlFlow> {
    let WhileLoop { condition, body } = while_loop;
    while eval_expr(condition, ctx)?.first_value().is_truthy() {
        if let ControlFlow::Return(ret_value) = eval_block(body, ctx)? {
            return Ok(ControlFlow::Return(ret_value));
        }
    }
    Ok(ControlFlow::Continue)
}

fn eval_block(block: &syn::Block, ctx: &mut EvalContext<'_>) -> Result<ControlFlow> {
    for statement in &block.statements {
        if let ControlFlow::Return(value) = eval_stmnt(statement, ctx)? {
            return Ok(ControlFlow::Return(value));
        }
    }
    block
        .ret
        .as_ref()
        .map(|ret| eval_ret(ret, ctx).map(ControlFlow::Return))
        .unwrap_or(Ok(ControlFlow::Continue))
}

fn eval_conditional(conditional: &syn::Conditional, ctx: &mut EvalContext) -> Result<ControlFlow> {
    let Conditional {
        condition,
        body,
        tail,
    } = conditional;

    if eval_expr(condition, ctx)?.first_value().is_truthy() {
        eval_block(body, ctx)
    } else {
        match tail {
            ConditionalTail::End => Ok(ControlFlow::Continue),
            ConditionalTail::Else(block) => eval_block(block, ctx),
            ConditionalTail::ElseIf(condition) => eval_conditional(condition, ctx),
        }
    }
}

fn eval_assignment(assignment: &syn::Assignment, ctx: &mut EvalContext) -> Result<()> {
    let Assignment { names, values } = assignment;
    assignment_values(ctx, values).and_then(|values| multiple_assignment(ctx, names, values))
}

fn assignment_values<'a>(
    ctx: &mut EvalContext,
    values: impl IntoIterator<Item = &'a Expression>,
) -> Result<impl Iterator<Item = LuaValue>> {
    values
        .into_iter()
        .map(|expr| eval_expr(expr, ctx))
        .collect::<Result<Vec<_>>>()
        .map(tail_values)
        .map(|values| values.chain(std::iter::repeat_with(|| LuaValue::Nil)))
}

fn multiple_assignment<'a>(
    ctx: &mut EvalContext,
    names: impl IntoIterator<Item = &'a Var>,
    values: impl Iterator<Item = LuaValue>,
) -> Result<()> {
    for (name, value) in names.into_iter().zip(values) {
        assign_to_var(name, value, ctx)?;
    }
    Ok(())
}

fn eval_ret(ret: &Return, ctx: &mut EvalContext) -> Result<ReturnValue> {
    match &ret.0[..] {
        [] => Ok(ReturnValue::NIL),
        // Common case optimization
        [expr] => eval_expr(expr, ctx),
        exprs => {
            let exprs = exprs
                .iter()
                .map(|expr| eval_expr(expr, ctx))
                .collect::<Result<Vec<_>>>()?;
            Ok(tail_values(exprs).collect())
        }
    }
}

fn eval_expr(expr: &Expression, ctx: &mut EvalContext) -> Result<ReturnValue> {
    match expr {
        Expression::Nil => Ok(ReturnValue::NIL),
        Expression::Number(NumberLiteral(num)) => Ok(ReturnValue::number(*num)),
        Expression::String(StringLiteral(str)) => Ok(ReturnValue::string(str)),
        Expression::Variable(var) => eval_var(var, ctx).map(ReturnValue::from),
        Expression::TableConstructor(tbl) => eval_tbl_constructor(tbl, ctx)
            .map(TableRef::from)
            .map(LuaValue::Table)
            .map(ReturnValue::from),
        Expression::FunctionCall(call) => eval_fn_call(call, ctx),
        Expression::UnaryOperator { op, exp } => {
            eval_unary_op_expr(exp.as_ref(), *op, ctx).map(ReturnValue::from)
        }
        Expression::BinaryOperator { lhs, op, rhs } => {
            binary_op_eval(*op, lhs, rhs, ctx).map(ReturnValue::from)
        }
    }
}

pub(crate) fn binary_op_eval(
    op: luar_syn::BinaryOperator,
    lhs: &Expression,
    rhs: &Expression,
    ctx: &mut EvalContext<'_>,
) -> Result<LuaValue> {
    use BinaryOperator::*;

    let lhs = eval_expr(lhs, ctx)?.first_value();
    match op {
        And if lhs.is_falsy() => return Ok(lhs),
        Or if lhs.is_truthy() => return Ok(lhs),
        _ => {}
    };

    let rhs = eval_expr(rhs, ctx)?.first_value();
    match op {
        Equals => return Ok(LuaValue::from_bool(lhs == rhs)),
        NotEquals => return Ok(LuaValue::from_bool(lhs != rhs)),
        And | Or => return Ok(rhs),
        _ => {}
    }

    match op {
        Less => less_than(lhs, rhs),
        Greater => greater_than(lhs, rhs),
        LessOrEquals => less_or_equals(lhs, rhs),
        GreaterOrEquals => greater_or_equals(lhs, rhs),
        Plus => binary_number_op(lhs, rhs, ArithmeticOperator::Add, std::ops::Add::add),
        Minus => binary_number_op(lhs, rhs, ArithmeticOperator::Sub, std::ops::Sub::sub),
        Mul => binary_number_op(lhs, rhs, ArithmeticOperator::Mul, std::ops::Mul::mul),
        Div => binary_number_op(lhs, rhs, ArithmeticOperator::Div, std::ops::Div::div),
        Exp => todo!("No support for ^ operator yet."),
        Concat => concat(lhs, rhs),
        And | Or | Equals | NotEquals => unreachable!(),
    }
    .map_err(EvalError::from)
}

fn eval_unary_op_expr(
    expr: &Expression,
    op: luar_syn::UnaryOperator,
    ctx: &mut EvalContext<'_>,
) -> Result<LuaValue> {
    eval_expr(expr, ctx).and_then(|value| {
        unary_op_eval(op, value.first_value())
            .map_err(TypeError::Arithmetic)
            .map_err(EvalError::from)
    })
}

fn eval_fn_call(call: &syn::FunctionCall, ctx: &mut EvalContext<'_>) -> Result<ReturnValue> {
    match call {
        FunctionCall::Function { func, args } => eval_fn_args(args, ctx).and_then(|args| {
            let fn_value = eval_var(func, ctx)?;
            call_value(ctx.context, &fn_value, &args)
        }),
        FunctionCall::Method { .. } => {
            todo!("Cannot evaluate method call yet")
        }
    }
}

fn eval_fn_args(args: &FunctionCallArgs, ctx: &mut EvalContext) -> Result<Vec<LuaValue>> {
    match args {
        FunctionCallArgs::Arglist(exprs) => exprs
            .into_iter()
            .map(|expr| eval_expr(expr, ctx))
            .map(|arg| arg.map(ReturnValue::first_value))
            .collect(),
        FunctionCallArgs::Table(table) => eval_tbl_constructor(table, ctx)
            .map(TableRef::from)
            .map(LuaValue::Table)
            .map(|table| vec![table]),
    }
}

fn eval_tbl_constructor(tbl: &syn::TableConstructor, ctx: &mut EvalContext) -> Result<TableValue> {
    let TableConstructor { lfield, ffield } = tbl;
    let mut table = TableValue::new();
    for (value, idx) in lfield.into_iter().zip(1usize..) {
        let key = LuaKey::number(idx);
        let value = eval_expr(value, ctx)?.first_value();
        table.set(key, value);
    }
    for (ident, expr) in ffield {
        let key = LuaKey::string(ident.as_ref());
        let value = eval_expr(expr, ctx)?.first_value();
        table.set(key, value);
    }

    Ok(table)
}

fn eval_var(var: &Var, ctx: &mut EvalContext) -> Result<LuaValue> {
    match var {
        Var::Named(id) => Ok(ctx.lookup(*id).clone()),
        Var::MemberLookup { from, value } => {
            let from = eval_var(from, ctx)?;
            let key = eval_expr(value, ctx)?.first_value();
            member_lookup(from, key)
        }
        Var::PropertyAccess { from, property } => {
            let from = eval_var(from, ctx)?;
            property_access(from, property.clone())
        }
    }
    .map_err(EvalError::from)
}

fn assign_to_var(var: &Var, value: LuaValue, ctx: &mut EvalContext) -> Result<()> {
    match var {
        Var::Named(id) => Ok(ctx.assign(*id, value)),
        Var::MemberLookup { from, value: key } => {
            let from = eval_var(from, ctx)?;
            let key = eval_expr(key, ctx)?.first_value();
            assign_to_value_member(from, key, value)
        }
        Var::PropertyAccess { from, property } => {
            let from = eval_var(from, ctx)?;
            assign_to_value_property(from, property.clone(), value)
        }
    }
    .map_err(EvalError::from)
}
