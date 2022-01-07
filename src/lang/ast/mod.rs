mod module;
mod ret;
mod expr;
mod stmnt;
mod fn_call;
mod var;

macro_rules! todo_eval {
    ($ret: ty, $name: ty) => {
        impl crate::lang::Eval for $name {
            type Return = $ret;

            fn eval(&self, _: &mut impl crate::lang::EvalContext) -> Result<Self::Return, crate::lang::EvalError> {
                todo!();
            }
        }
    };
}

todo_eval!(crate::lang::LuaValue, crate::syn::TableConstructor);
todo_eval!((), crate::syn::FunctionDeclaration);