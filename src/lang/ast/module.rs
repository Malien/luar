use crate::{lang::{Eval, LuaValue, EvalContext, EvalError}, syn::{Module, Chunk}};

impl Eval for Module {
    type Return = LuaValue;

    fn eval(&self, context: &mut impl EvalContext) -> Result<LuaValue, EvalError> {
        for chunk in &*self.chunks {
            chunk.eval(context)?;
        }
        match self.ret {
            Some(ref ret) => ret.eval(context),
            None => Ok(LuaValue::Nil),
        }
    }
}

impl Eval for Chunk {
    type Return = ();

    fn eval(&self, context: &mut impl EvalContext) -> Result<(), EvalError> {
        match self {
            Chunk::Statement(stmnt) => stmnt.eval(context),
            Chunk::FnDecl(decl) => decl.eval(context),
        }
    }
}
