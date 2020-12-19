use indoc::indoc;
use logos::Logos;

mod lex;

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
    println!("{}", LUA_FUNCTION);
    let lexer = lex::Token::lexer(LUA_FUNCTION);
    for lexem in lexer {
        println!("{:?}", lexem);
    }
}
