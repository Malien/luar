pub mod ast_vm_test_harness;
// pub mod reggie_test_harness;

mod ast_vm {
    use crate::ast_vm_test_harness::run_lua_test;

    macro_rules! lua_test {
        ($name: ident) => {
            #[test]
            fn $name() {
                run_lua_test(
                    stringify!($name),
                    include_str!(concat!("./", stringify!($name), ".test.lua")),
                );
            }
        };
    }

    lua_test!(conditional);
    lua_test!(assignment);
}
