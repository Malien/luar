use crate::{
    lang::{ControlFlow, Eval, EvalContext, EvalError, LuaValue},
    syn::{Chunk, Module},
};

impl Eval for Module {
    type Return = LuaValue;

    fn eval<Context>(&self, context: &mut Context) -> Result<LuaValue, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        for chunk in &*self.chunks {
            if let ControlFlow::Return(value) = chunk.eval(context)? {
                return Ok(value);
            }
        }
        match self.ret {
            Some(ref ret) => ret.eval(context),
            None => Ok(LuaValue::Nil),
        }
    }
}

impl Eval for Chunk {
    type Return = ControlFlow;

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        match self {
            Chunk::Statement(stmnt) => stmnt.eval(context),
            Chunk::FnDecl(decl) => decl.eval(context).map(|_| ControlFlow::Continue),
        }
    }
}
