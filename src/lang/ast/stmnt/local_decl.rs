use crate::{
    lang::{Eval, EvalContext, EvalContextExt, LuaValue},
    lex::Ident,
    syn::Declaration,
};

use super::assignment::assignment_values;

impl Eval for Declaration {
    type Return = ();

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, crate::lang::EvalError>
    where
        Context: crate::lang::EvalContext + ?Sized,
    {
        let Self {
            names,
            initial_values,
        } = self;
        assignment_values(context, initial_values)
            .map(|values| multiple_local_assignment(context, names.clone(), values))
    }
}

fn multiple_local_assignment<'a, Context: EvalContext + ?Sized>(
    context: &mut Context,
    names: impl IntoIterator<Item = Ident>,
    values: impl Iterator<Item = LuaValue>,
) {
    for (name, value) in names.into_iter().zip(values) {
        context.set(name, value.first_value());
    }
}

#[cfg(test)]
mod test {
    use crate::{
        error::LuaError,
        lang::{Eval, EvalContextExt, GlobalContext, LuaValue},
        lex::Ident,
        syn::string_parser,
    };

    #[quickcheck]
    fn local_decl_behaves_like_global_assignment_in_global_scope(
        ident: Ident,
        value: LuaValue,
    ) -> Result<(), LuaError> {
        let module = string_parser::module(&format!("local {} = value", ident))?;
        let mut context = GlobalContext::new();
        context.set("value", value.clone());
        module.eval(&mut context)?;
        assert!(context.get(&ident).total_eq(&value));
        Ok(())
    }
}
