# Squiid Engine

Squiid Engine is the core computational engine for [Squiid Calculator](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid), designed for efficient and precise mathematical computations using Reverse Polish Notation (RPN). It is implemented in Rust to ensure high performance and safety.

This library serves as the backend for Squiid Calculator, handling all mathematical operations, stack management, and evaluation logic. While Squiid Engine natively operates on RPN expressions, it can be paired with [squiid-parser](https://crates.io/crates/squiid-parser) for converting algebraic expressions into RPN.

## Features

- Fast and memory-safe RPN evaluation
- Strongly typed and well-documented API
- Error handling with the prohibition of `unwrap()`/`expect()`
- Able to be used through a variety of different languages through the [bindings](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid-bindings)

## Installation

```sh
cargo add squiid-engine
```

or, manually add to your Cargo.toml:

```toml
[dependencies]
squiid-engine = "0.1"
```

## Example Usage

```rs
use squiid_engine::SquiidEngine;

fn main() {
let mut engine = SquiidEngine::new();

    // Execute an RPN expression: 3 + (5 * 7)
    let result = engine.execute_multiple_rpn(&["3", "5", "7", "multiply", "add"]);

    assert!(result.is_ok());

    let stack = engine.get_stack();
    assert_eq!(stack[0].value, "38");

}
```

## License

Squiid Engine is licensed under GPLv3.

# Compiling

To compile the shared object file for use in bindings, just run install Rust and run `cargo build --release --lib --features=ffi`. The shared object file should be found in `target/release/libsquiid_engine.so`.
