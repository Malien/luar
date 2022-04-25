use std::panic::{catch_unwind, resume_unwind};

use luar_syn::lua_parser;
use reggie::{eval_module, stdlib, LuaValue, Machine, NativeFunction, call_block};

macro_rules! run_lua_test {
    ($group_name: expr, $module_str: expr) => {
        $crate::reggie_test_harness::run_lua_test_impl(module_path!(), $group_name, $module_str);
    };
}

pub(crate) use run_lua_test;

pub fn run_lua_test_impl(module_path: &str, group_name: &str, module_str: &str) {
    let res = catch_unwind(|| {
        let mut machine = Machine::new();
        machine.global_values.set(
            "assert",
            LuaValue::NativeFunction(NativeFunction::new(stdlib::assert)),
        );
        let module = lua_parser::module(module_str).unwrap();
        let res = eval_module::<()>(&module, &mut machine);
        if let Err(err) = res {
            eprintln!("Error occurred while evaluating test module: {}", err);
            return true;
        }

        let test_cases: Vec<_> = (&machine.global_values)
            .into_iter()
            .map(|value| (value.name.clone(), value.value.clone()))
            .filter_map(|(name, value)| LuaValue::as_lua_function(value).map(|func| (name, func)))
            .filter(|(name, _)| *name != "assert" && !name.starts_with('_'))
            .collect();
        let mut error_occurred = false;

        for (name, func) in test_cases {
            let res = call_block::<()>(func, &mut machine);
            match res {
                Ok(_) => println!("✅ {}::{}::{}", module_path, group_name, name),
                Err(err) => {
                    error_occurred = true;
                    eprintln!("❌ {}::{}::{}\n\t{}", module_path, group_name, name, err);
                }
            }
        }
        
        error_occurred
    });

    match res {
        Ok(true) => {
            panic!("Error occurred while running lua test group {}", group_name);
        }
        Err(err) => {
            eprintln!("Test group {} panicked during execution", group_name);
            resume_unwind(err);
        }
        Ok(false) => {}
    }
}
