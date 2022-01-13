use crate::{
    lang::{Eval, EvalContext, EvalError},
    syn::Statement,
};

mod assignment;
mod local_decl;

impl Eval for Statement {
    type Return = ();

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        match self {
            Self::Assignment(assignment) => assignment.eval(context),
            Self::LocalDeclaration(decl) => decl.eval(context),
            Self::FunctionCall(func_call) => func_call.eval(context).map(|_| ()),
            _ => todo!(),
        }
    }
}
