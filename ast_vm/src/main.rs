use ast_vm::{
    lang::{EvalError, LuaValue, TypeError},
    stdlib, LuaError,
};
use luar_syn::{lua_parser, ParseError, ParseErrorWithSourcePosition, RawParseError};
use std::{collections::HashSet, error::Error, mem::size_of};

fn repl() -> Result<(), Box<dyn Error>> {
    use std::io::{BufRead, Write};

    let mut context = stdlib::std_context();
    print!(">>> ");
    std::io::stdout().flush()?;
    for line in std::io::stdin().lock().lines() {
        let module = lua_parser::module(&line?)?;
        let res = ast_vm::eval_module(&module, &mut context);
        match res {
            Ok(value) => println!("{}", value),
            Err(err) => println!("Error: {}", err),
        }
        print!(">>> ");
        std::io::stdout().flush()?;
    }
    Ok(())
}

fn eval_file(filename: &str) -> Result<(), Box<dyn Error>> {
    use std::io::Read;

    let mut file = std::fs::File::open(filename)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    let module = lua_parser::module(&buffer)?;
    ast_vm::eval_module(&module, &mut stdlib::std_context())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("LuaError: {}", size_of::<LuaError>());
    println!("EvalError: {}", size_of::<EvalError>());
    println!("ParseError: {}", size_of::<ParseError>());
    println!("TypeError: {}", size_of::<TypeError>());
    println!("LuaValue: {}", size_of::<LuaValue>());
    println!("HashSet: {}", size_of::<HashSet::<&'static str>>());
    println!(
        "ParseErrorWithSourcePosition: {}",
        size_of::<ParseErrorWithSourcePosition>()
    );
    println!("RawParseError: {}", size_of::<RawParseError>());

    if let Some(filename) = std::env::args().skip(1).next() {
        eval_file(&filename)
    } else {
        repl()
    }
}
