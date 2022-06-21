use crate::{
    compiler::compile_function,
    ids::{ArgumentRegisterID, LocalRegisterID},
    machine::CodeBlock,
    ops::Instruction,
    GlobalValues,
};

pub fn optimize(block: &CodeBlock) -> CodeBlock {
    block.clone()
}

#[test]
fn optimization_result() {
    let decl = luar_syn::lua_parser::function_declaration(
        "
        function foo(a)
            local b = a + 1
            local c = a + 2
            return c
        end
    ",
    )
    .unwrap();
    let input = compile_function(&decl, &mut GlobalValues::default());
    println!("input: {}\n", input);
    println!("output: {}", optimize(&input));
}
