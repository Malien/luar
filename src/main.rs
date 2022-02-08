use std::{error::Error, io::{BufRead, Write}};

use indoc::indoc;

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

#[allow(unused)]
static LUA_FUNCTION: &'static str = indoc! {"
    function remove_blanks (s)
        local b = strfind(s, ' ')
        while b do
            s = strsub(s, 1, b-1) .. strsub(s, b+1)
            b = strfind(s, ' ')
        end
        return s
    end
"};

// static ACKERMAN_BENCH: &str = include_str!("../benchmarks/ack.lua");

fn main() -> Result<(), Box<dyn Error>> {
    // let tokens: Vec<_> = lex::Token::lexer(LUA_FUNCTION).collect();
    // let parsed = syn::lua_parser::module(&tokens).unwrap();
    // println!("{}\n{:#?}", parsed, parsed);
    let mut context = lang::GlobalContext::new();
    // let tokens =
    // let tokens: Vec<_> = lex::Token::lexer(ACKERMAN_BENCH).collect();
    // println!("{:#?}", ACKERMAN_BENCH);
    // let module = syn::string_parser::module(ACKERMAN_BENCH).unwrap();
    // let res = lang::Eval::eval(&module, &mut context).unwrap();
    // println!("{}", res);
    print!(">>> ");
    std::io::stdout().flush()?;
    for line in std::io::stdin().lock().lines() {
        let module = syn::string_parser::module(&line?)?;
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
