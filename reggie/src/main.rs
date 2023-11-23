use itertools::Itertools;
use reggie::{eval_str, LuaValue, Machine};
use std::error::Error;

fn repl() -> Result<(), Box<dyn Error>> {
    use std::io::{BufRead, Write};

    let mut machine = Machine::with_stdlib();
    print!(">>> ");
    std::io::stdout().flush()?;
    for line in std::io::stdin().lock().lines() {
        let res = eval_str::<&[LuaValue]>(&line?, &mut machine);
        match res {
            Ok(values) if values.len() > 0 => println!("{}", values.iter().join("\t")),
            Ok(_) => {},
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
    eval_str::<()>(&buffer, &mut Machine::with_stdlib())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let zeros = [0u8; std::mem::size_of::<LuaValue>()];
    let zeros: LuaValue = unsafe { std::mem::transmute(zeros) };
    println!("LuaValue: {zeros:?}");
    if let Some(filename) = std::env::args().skip(1).next() {
        eval_file(&filename)
    } else {
        repl()
    }
}
