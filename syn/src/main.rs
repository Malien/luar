use luar_syn::lua_parser;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    use std::io::{BufRead, Write};

    print!(">>> ");
    std::io::stdout().flush()?;
    for line in std::io::stdin().lock().lines() {
        let res = lua_parser::module(&line?);
        println!("{:#?}", res);
        std::io::stdout().flush()?;
    }
    Ok(())
}
