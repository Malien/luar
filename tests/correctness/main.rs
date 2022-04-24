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
                use reggie::{eval_str, stdlib, LuaValue, Machine, NativeFunction};

                let mut machine = Machine::new();
                machine.global_values.set(
                    "assert",
                    LuaValue::NativeFunction(NativeFunction::new(stdlib::assert)),
                );
                eval_str(TEST_CODE, &mut machine)?;
                Ok(())
            }
        }
    };
    ($($name: ident),*) => {
        mod ast_vm {
            use luar_syn::lua_parser;

            $(
                #[test]
                fn $name() -> Result<(), ast_vm::LuaError> {
                    let test_code = include_str!(concat!("./", stringify!($name), ".test.lua"));
                    let module = lua_parser::module(test_code)?;
                    let mut context = ast_vm::stdlib::std_context();
                    ast_vm::eval_module(&module, &mut context)?;
                    Ok(())
                }
            )+
        }

        mod reggie {
            use reggie::{eval_str, stdlib, LuaValue, Machine, NativeFunction};
            $(
                #[test]
                fn $name() -> Result<(), reggie::LuaError> {

                    let test_code = include_str!(concat!("./", stringify!($name), ".test.lua"));
                    let mut machine = Machine::new();
                    machine.global_values.set(
                        "assert",
                        LuaValue::NativeFunction(NativeFunction::new(stdlib::assert)),
                    );
                    eval_str(test_code, &mut machine)?;
                    Ok(())
                }
            )*
        }
    };
}

test_case![heapsort, fib_rec, fib_tailrec];