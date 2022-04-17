pub use luar::ast_vm::AstVM;

pub mod ast_vm_test_harness;
pub mod basic_expr;

mod ast_vm {
    use crate::{ast_vm_test_harness::run_lua_test, basic_expr_tests};

    basic_expr_tests!(luar::ast_vm::AstVM, luar::lang::GlobalContext::new());

    #[test]
    fn lua_tests() {
        run_lua_test("conditional", include_str!("./conditional.test.lua"));
    }
}

mod reggie {
    use crate::basic_expr_tests;

    basic_expr_tests!(luar::ast_vm::AstVM, luar::lang::GlobalContext::new());
}

// tests!(ast_vm, luar::ast_vm::AstVM, ::luar::lang::GlobalContext::new());
// tests!(reggie, luar::reggie::ReggieVM, ::luar::reggie::Machine::new());
