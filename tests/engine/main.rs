pub mod ast_vm_test_harness;
pub mod reggie_test_harness;

macro_rules! lua_test {
    ($test_fn: tt !, $name: ident) => {
        #[test]
        fn $name() {
            $test_fn!(
                stringify!($name),
                include_str!(concat!("./", stringify!($name), ".test.lua"))
            );
        }
    };
    ($test_fn: tt !, [$($name: ident),*$(,)?]) => {
        $(lua_test!($test_fn!, $name);)*
    }
}

macro_rules! run_tests {
    ($test_fn: path) => {
        lua_test!($test_fn!, [
            conditional,
            assignment,
            local_decl,
            while_loop,
            tables,
            arithmetic,
            comparison,
        ]);
    };
}


mod ast_vm {
    run_tests!(crate::ast_vm_test_harness::run_lua_test);
}

mod reggie {
    run_tests!(crate::reggie_test_harness::run_lua_test);
}
