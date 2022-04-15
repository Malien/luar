pub use luar::ast_vm::AstVM;

pub mod basic_expr;
pub mod conditional;

macro_rules! tests {
    ($name: ident, $engine: ty, $context: expr) => {
        mod $name {
            use crate::{basic_expr_tests, conditional_tests};

            basic_expr_tests!($engine, $context);
            conditional_tests!($engine, $context);
        }
    };
}

tests!(ast_vm, luar::ast_vm::AstVM, ::luar::lang::GlobalContext::new());
tests!(reggie, luar::reggie::ReggieVM, ::luar::reggie::Machine::new());
