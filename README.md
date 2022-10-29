# Luar — an attempt to make an optimizing lua compiler

The project consists of several crates:
- `lex` houses the lexer and lexing constructs, such as `StringLiteral`, `NumberLiteral` and `Ident`
- `syn` contains the parser and language constructs definition as rust structs
- `error` a hierarchy of error types common in different lua runtimes
- `ast_vm` a runtime that executes AST coming directly from `syn`
- `reggie` a register based VM and an optimizing compiler
- `non_empty` a Vec that cannot be empty
- `test_util` just a few helper structs/functions for testing
- `tests` integration tests and benchmarks

### Running the thing
- `cargo run --bin reggie` to launch REPL of register based VM (relevant version)
- `cargo run --bin reggie <filename>` to execute file in register based VM
- `cargo run --bin ast_vm` to launch REPL of AST interpretor
- `cargo run --bin ast_vm <filename` to execute file in AST interpretor
