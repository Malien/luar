pub mod ast_vm_test_harness;
pub mod reggie_test_harness;

macro_rules! lua_test {
    ($test_fn: path, $name: ident) => {
        #[test]
        fn $name() {
            $test_fn(
                stringify!($name),
                include_str!(concat!("./", stringify!($name), ".test.lua")),
            );
        }
    };
}

macro_rules! run_tests {
    ($test_fn: path) => {
        lua_test!($test_fn, conditional);
        lua_test!($test_fn, assignment);
        lua_test!($test_fn, local_decl);
    };
}


mod ast_vm {
    run_tests!(crate::ast_vm_test_harness::run_lua_test);
}

mod reggie {
    run_tests!(crate::reggie_test_harness::run_lua_test);
}
