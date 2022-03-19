use std::error::Error;

use luar::{lang::ast, stdlib, syn};

fn repl() -> Result<(), Box<dyn Error>> {
    use std::io::{BufRead, Write};

    let mut context = stdlib::std_context();
    print!(">>> ");
    std::io::stdout().flush()?;
    for line in std::io::stdin().lock().lines() {
        let module = syn::lua_parser::module(&line?)?;
        let res = ast::eval_module(&module, &mut context);
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
    let module = syn::lua_parser::module(&buffer)?;
    ast::eval_module(&module, &mut stdlib::std_context())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    if let Some(filename) = std::env::args().skip(1).next() {
        eval_file(&filename)
    } else {
        repl()
    }
}
