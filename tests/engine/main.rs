pub mod ast_vm_test_harness;
pub mod basic_expr;

mod ast_vm {
    use crate::{ast_vm_test_harness::run_lua_test, basic_expr_tests};

    basic_expr_tests!(ast_vm::ast_vm::AstVM, ast_vm::lang::GlobalContext::new());

    macro_rules! lua_test {
        ($name: ident) => {
            #[test]
            fn $name() {
                run_lua_test(stringify!($name), include_str!(concat!("./", stringify!($name), ".test.lua")));
            }
        };
    }

    lua_test!(conditional);
    lua_test!(assignment);
}

mod reggie {
    // use crate::basic_expr_tests;

    // basic_expr_tests!(ast_vm::AstVM, ast_vm::lang::GlobalContext::new());
}

// tests!(ast_vm, luar::ast_vm::AstVM, ::luar::lang::GlobalContext::new());
// tests!(reggie, luar::reggie::ReggieVM, ::luar::reggie::Machine::new());
