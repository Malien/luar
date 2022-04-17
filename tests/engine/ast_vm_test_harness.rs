use luar::ast_vm::eval_module;
use luar::lang::{GlobalContext, LuaValue, ReturnValue};
use luar::stdlib::fns;
use luar_syn::lua_parser;

pub fn run_lua_test(group_name: &str, module_str: &str) {
    let mut context = GlobalContext::new();
    context.set(
        "assert",
        LuaValue::function(move |_, args| fns::assert(args).map(ReturnValue::from)),
    );
    let module = lua_parser::module(module_str).unwrap();
    eval_module(&module, &mut context).unwrap();

    let test_cases: Vec<_> = (&context)
        .into_iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .filter_map(|(name, value)| LuaValue::as_function(value).map(|func| (name, func)))
        .filter(|(name, _)| *name != "assert" && !name.starts_with('_'))
        .collect();
    let mut error_occurred = false;

    for (name, func) in test_cases {
        let res = func.call(&mut context, &[]);
        match res {
            Ok(_) => println!("✅ {}::{}", group_name, name),
            Err(err) => {
                error_occurred = true;
                println!("❌ {}::{}\n\t{}", group_name, name, err);
            }
        }
    }

    if error_occurred {
        panic!("Error occurred while running lua test group {}", group_name);
    }
}
