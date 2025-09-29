# Hades

> A statically typed, systems programming language with LLVM backend

Hades is a toy, statically typed programming language built with Rust and LLVM in the backend(WIP).

## Current Status
**Phase**: Early Development (Parser & AST Complete)

The Hades compiler currently includes:
- Lexical analysis (tokenizer)
- Syntax parsing with error recovery
- Abstract Syntax Tree (AST) generation
- Basic error reporting with diagnostics

## TODO List

### Completed
- [x] **Lexer**: Tokenize source code into language tokens
- [x] **Parser**: Build Abstract Syntax Tree from tokens
- [x] **AST**: Define and implement language constructs
- [x] **Error Reporting**: Basic diagnostic messages with source location

### In Progress / Remaining
- [ ] **Semantic Analysis**: Type checking and symbol resolution
- [ ] **LLVM Integration**: Set up LLVM bindings for code generation
- [ ] **Code Generation**: Generate LLVM IR from AST
- [ ] **Standard Library**: Implement basic built-in functions
- [ ] **Memory Management**: Design allocation and deallocation strategy
- [ ] **Optimization**: Basic LLVM optimization passes

