use indoc::indoc;
use logos::Logos;

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[cfg(test)]
mod test_util;

mod lex;
mod syn;
mod lang;
mod util;
mod error;

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

fn main() {
    let tokens: Vec<_> = lex::Token::lexer(LUA_FUNCTION).collect();
    let parsed = syn::lua_parser::module(&tokens).unwrap();
    println!("{}\n{:#?}", parsed, parsed);
}
