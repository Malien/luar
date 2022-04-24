use std::io::Read;

use luar_syn::lua_parser;
use reggie::{GlobalValues, compiler::compile_module};

fn main() {
    let mut buf = String::new();
    std::io::stdin().lock().read_to_string(&mut buf).unwrap();
    let mut global_values = GlobalValues::default();
    let module = lua_parser::module(&buf).unwrap();
    let compiled_module = compile_module(&module, &mut global_values);
    println!("{}", global_values);
    println!("{}", compiled_module);
}