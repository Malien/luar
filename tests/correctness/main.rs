macro_rules! test_case {
    ($name: ident) => {
        mod $name {
            use luar_syn::lua_parser;
            use luar::{ast_vm, error::LuaError, stdlib::std_context, reggie};

            static TEST_CODE: &str = include_str!(concat!("./", stringify!($name), ".test.lua"));

            #[test]
            fn ast_vm() -> Result<(), LuaError> {
                let module = lua_parser::module(TEST_CODE)?;
                let mut context = std_context();
                ast_vm::eval_module(&module, &mut context)?;
                Ok(())
            }

            #[test]
            fn reggie() -> Result<(), LuaError> {
                let mut machine = reggie::Machine::new();
                reggie::eval_str(TEST_CODE, &mut machine)?;
                Ok(())
            }
        }
    };
    ($($rest: ident),+) => {
        $(test_case!($rest);)+
    }
}

test_case![heapsort, fib_rec];
