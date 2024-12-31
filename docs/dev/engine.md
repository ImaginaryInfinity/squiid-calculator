# Engine Documentation

The Squiid engine is a Rust module that defines the backend functionality for Squiid. The engine handles all of the computation of RPN expressions, as well as state for things like variables, the stack, and anything else you would need in a calculator.

## Main engine functionality

The function also maintains a history of the calculator's state, including the stack and variable states. This history is used to implement the undo command, which restores the calculator's state to a previous point in time.

The engine can either be used as a Rust module in Rust programs, or as a shared library over FFI. We provide some language bindings for other languages to easily use the engine [here](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid-bindings). `lib.rs` is the main point of where you would interact with the engine. It provides the `execute_multiple_rpn` and `execute_single_rpn` functions to easily execute RPN expressions, and functions like `get_stack`, `get_commands`, and `get_previous_answer` help to query the state of the engine.

The quit command can be used to signal an intent to exit.

Overall, this code provides the core functionality for a command-line calculator.

!!! important

    Since the engine has no real idea of what an "expression" is in the sense of algebraic equations, frontends need to manually call `update_previous_answer` after a full algebraic expression has been submitted to the engine with the `execute_*_rpn` functions.

## Internal files

The code consists of the following modules:

<!-- TODO: additional documentation for each file and it's public functions -->

### `bucket.rs`

This module contains the definition of a bucket, which is a stack-based data structure used to store operands and operators in Squiid.

### `command_mappings.rs`

This module defines a mapping between the commands supported by the calculator and the corresponding functions that are used to evaluate them.

### `engine.rs`

This module contains the main logic for the calculator backend, including the stack, history, and variable states.

### `ffi/`

This directory provides an interface to other programming languages, allowing the calculator to be used as a dynamic library. We have provided some premade bindings to languages such as Python, C, and C++. These bindings, as well as examples on how to use them, can be found at <https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid-bindings/>

### `utils.rs`

This module contains utility functions for sending and receiving data over the network.

