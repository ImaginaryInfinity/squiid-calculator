# Engine Documentation

The Squiid engine is a Rust module that defines the backend server for Squiid. The server listens for incoming requests from clients, evaluates the mathematical expressions sent by the clients, and sends back the results.

## Main engine functionality
The `start_server` function is the entry point for the module. It takes an optional address parameter that specifies the address to bind the server to. If no address is provided, it binds to the default address `tcp://*:33242`.

The function creates an NNG socket and binds it to the specified address. It then creates an instance of the Engine struct, which is used to evaluate expressions. It also creates a mapping between the commands supported by the calculator and the corresponding functions.

The server then enters an infinite loop, where it waits for incoming requests from clients. When a request is received, the server parses the JSON data sent by the client and evaluates the expression. If the evaluation succeeds, the server sends back the result to the client. If the evaluation fails, the server sends an error message back to the client.

The function also maintains a history of the calculator's state, including the stack and variable states. This history is used to implement the undo command, which restores the calculator's state to a previous point in time.

The quit command can be used to exit the server.

The `handle_data` file is an abstraction which allows the calculator to be run without NNG if wanted. This could be useful in certain cases such as WebAssembly where we can't use IPC to communicate between the frontend and backend. If you would like to disable NNG, just include squiid engine without any default features in your Rust project. The `ipc` feature is what adds IPC support. When you are not using the included `start_server` function, you will need to maintain engine state and communication between client and server yourself, which shouldn't be too difficult. Check `lib.rs` for an example on how to do that.

Overall, this code provides the core functionality for a command-line calculator server. It can be used as a library in other programs (as a Rust library or as a shared object file) or as a standalone calculator server.

## Internal files
The code consists of the following modules:

<!-- TODO: additional documentation for each file and it's public functions -->

### `bucket.rs`
This module contains the definition of a bucket, which is a stack-based data structure used to store operands and operators in Squiid.

### `command_mappings.rs`
This module defines a mapping between the commands supported by the calculator and the corresponding functions that are used to evaluate them.

### `engine.rs`
This module contains the main logic for the calculator backend, including the stack, history, and variable states.

### `ffi.rs`
This module provides an interface to other programming languages, allowing the calculator to be used as a shared object library. <!-- TODO: move to separate file--> Here's an example program in C:
```c
// compiled with `gcc test.c -o test -L. -l:libsquiid_engine.so`
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

// Define function signatures of Rust functions
extern char** start_server_exposed(const char* input, bool blocking);

int main() {
    // create the string to parse
    const char* input = "tcp://*:33242";

    // call Rust function from shared object
    start_server_exposed(input, true);
    return 0;
}
```

Additional language bindings and examples can be found at [https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid-bindings](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid-bindings).

### `protocol/client_request.rs`
This module defines the JSON protocol used to communicate from the client to the server.

### `protocol/server_response.rs`
This module defines the JSON protocol used to communicate from the server to the client.

### `utils.rs`
This module contains utility functions for sending and receiving data over the network.