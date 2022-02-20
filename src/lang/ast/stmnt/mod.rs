use crate::{
    lang::{ControlFlow, Eval, EvalContext, EvalError},
    syn::Statement,
};

mod assignment;
mod conditional;
mod local_decl;
mod while_loop;

impl Eval for Statement {
    type Return = ControlFlow;

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        match self {
            Self::Assignment(assignment) => assignment.eval(context).map(|_| ControlFlow::Continue),
            Self::LocalDeclaration(decl) => decl.eval(context).map(|_| ControlFlow::Continue),
            Self::FunctionCall(func_call) => func_call.eval(context).map(|_| ControlFlow::Continue),
            Self::If(conditional) => conditional.eval(context),
            Self::While(while_loop) => while_loop.eval(context),
            _ => todo!(),
        }
    }
}
