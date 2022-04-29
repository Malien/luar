macro_rules! test_case {
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
            use reggie::{eval_str, stdlib::define_stdlib, Machine};
            $(
                #[test]
                fn $name() -> Result<(), reggie::LuaError> {

                    let test_code = include_str!(concat!("./", stringify!($name), ".test.lua"));
                    let mut machine = Machine::new();
                    define_stdlib(&mut machine.global_values);
                    eval_str(test_code, &mut machine)?;
                    Ok(())
                }
            )*
        }
    };
}

test_case![heapsort, fib_rec, fib_tailrec, fib_loop];
