macro_rules! test_case {
    ($name: ident) => {
        mod $name {
            use luar_syn::lua_parser;

            static TEST_CODE: &str = include_str!(concat!("./", stringify!($name), ".test.lua"));

            #[test]
            fn ast_vm() -> Result<(), ast_vm::LuaError> {
                let module = lua_parser::module(TEST_CODE)?;
                let mut context = ast_vm::stdlib::std_context();
                ast_vm::eval_module(&module, &mut context)?;
                Ok(())
            }

            #[test]
            fn reggie() -> Result<(), reggie::LuaError> {
                use reggie::{Machine, eval_str, LuaValue, NativeFunction, stdlib};

                let mut machine = Machine::new();
                machine.global_values.set(
                    "assert",
                    LuaValue::NativeFunction(NativeFunction::new(stdlib::assert))
                );
                eval_str(TEST_CODE, &mut machine)?;
                Ok(())
            }
        }
    };
    ($($rest: ident),+) => {
        $(test_case!($rest);)+
    }
}

test_case![heapsort, fib_rec];
