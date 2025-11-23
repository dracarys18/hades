# Hades
Hades is a statically typed systems programming language which uses LLVM as compiler backend. It aims to provide simple C like syntax with some static analysis so you don't shoot yourself in the foot

## Getting Started

### Prerequisites

You'll need the following installed on your system:

- Rust (latest stable version)
- LLVM 15 or later

### Building from Source

Clone the repository and build the compiler:

```bash
git clone https://github.com/dracarys18/hades.git
cd hades
cargo build --release
```

The compiled binary will be available at `target/release/hades`.

### Examples
Examples are available in `examples/` directory.

## Project Status

The Hades compiler is currently in early development. Here's what's working:
- [x] Lexer
- [x] Parser
- [x] Code generation
- [x] Semantic analysis and type checking

The language is functional enough to write simple programs, but many features are still being developed. Expect breaking changes as the language evolves.

## Contributing

Hades is an open source project and contributions are welcome. Whether you want to fix bugs, add features, or improve documentation, your help is appreciated.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
