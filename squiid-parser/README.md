# Squiid Parser

Squiid Parser is the algebraic expression parser for [Squiid Calculator](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid), designed to parse infix notation into postifix (RPN) notation, which can then be evaluated by [squiid-engine](https://crates.io/crates/squiid-engine). It is implemented in Rust to ensure high performance and safety.

## Features

- Fast and memory-safe algebraic (infix) to RPN (postfix) conversion
- Well-documented API
- Error handling with the prohibition of `unwrap()`/`expect()`
- Able to be used through a variety of different languages through the [bindings](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid-bindings)

## Installation

```sh
cargo add squiid-parser
```

## Example Usage

```rs
use squiid_parser::parse;

fn main() {
    // Parse an algebraic expression: (3 + 5 * 7)
    let result = parse("(3+5*7)");

    assert_eq!(result, vec!["3", "5", "7", "*", "+"]);
}
```

## License

Squiid Parser is licensed under GPLv3.

# Compiling

To compile the shared object file for use in bindings, just run install Rust and run `cargo build --release --lib --features=ffi`. The shared object file should be found in `target/release/libsquiid_parser.so`.
