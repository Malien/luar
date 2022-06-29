use std::collections::HashSet;

use ast_vm::{eval_module, lang::LuaValue, stdlib};
use luar_syn::lua_parser;

macro_rules! run_lua_test {
    ($group_name: expr, $module_str: expr) => {
        $crate::ast_vm_test_harness::run_lua_test_impl(module_path!(), $group_name, $module_str);
    };
}

pub(crate) use run_lua_test;

pub fn run_lua_test_impl(module_path: &str, group_name: &str, module_str: &str) {
    let mut context = stdlib::std_context();

    let already_defined_fns: HashSet<_> = context.iter().map(|(a, _)| a).cloned().collect();

    let module = lua_parser::module(module_str).unwrap();
    eval_module(&module, &mut context).unwrap();

    let test_cases: Vec<_> = (&context)
        .into_iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .filter_map(|(name, value)| LuaValue::as_function(value).map(|func| (name, func)))
        .filter(|(name, _)| !already_defined_fns.contains(name) && !name.starts_with('_'))
        .collect();
    let mut error_occurred = false;

    for (name, func) in test_cases {
        let res = func.call(&mut context, &[]);
        match res {
            Ok(_) => println!("✅ {}::{}::{}", module_path, group_name, name),
            Err(err) => {
                error_occurred = true;
                println!("❌ {}::{}::{}\n\t{}", module_path, group_name, name, err);
            }
        }
    }

    if error_occurred {
        panic!("Error occurred while running lua test group {}", group_name);
    }
}
