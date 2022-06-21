# Luar — an attempt to make an optimizing lua compiler

The project consists of several crates:
- `lex` houses the lexer and lexing constructs, such as `StringLiteral`, `NumberLiteral` and `Ident`
- `syn` contains the parser and language constructs definition as rust structs
- `error` a hierarchy of error types common in different lua runtimes
- `ast_vm` a runtime that executes AST coming directly from the parser (`syn`)
- `reggie` a register based VM and an optimizing compiler
- `non_empty` a Vec that cannot be empty
- `test_util` just a few helper structs/functions for testing
- `tests` tests and benchmarks
