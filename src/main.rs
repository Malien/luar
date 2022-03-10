use std::error::Error;

use crate::lang::Eval;

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[cfg(test)]
mod test_util;

pub mod error;
pub mod lang;
pub mod lex;
pub mod stdlib;
pub mod syn;
mod util;

fn repl() -> Result<(), Box<dyn Error>> {
    use std::io::{Write, BufRead};

    let mut context = stdlib::std_context();
    print!(">>> ");
    std::io::stdout().flush()?;
    for line in std::io::stdin().lock().lines() {
        let module = syn::lua_parser::module(&line?)?;
        let res = module.eval(&mut context);
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
    module.eval(&mut stdlib::std_context())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    if let Some(filename) = std::env::args().skip(1).next() {
        eval_file(&filename)
    } else {
        repl()
    }
}
