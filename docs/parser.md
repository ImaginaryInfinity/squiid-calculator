# Parser Documentation

The Squiid Parser is a Rust implementation of a mathematical expression parser, with support for implicit multiplication and negative signs. The parser is based on the Shunting Yard algorithm, which is used to convert mathematical expressions written in infix notation to postfix notation. The parser takes in a vector of tokens and returns a vector of string references.

## Functions

### `lib::parse`

This function lexes and parses a given string automatically. You won't need to call any other internal methods when this is run, as all required operations will be performed. 

### `lexer::lex`

This function automatically lexes a given input string into a `Result<Vec<Token>, String>` using the `logos` crate. This is meant to be used internally however it may be used externally if needed. No parsing is done to the lexed string.

### `parser::shunting_yard_parser`

This function takes a vector of `Token`s and returns a vector of string references. It uses an implementation of the Shunting Yard algorithm to parse the expression. The function creates two stacks, `output_queue` and `operator_stack`, to keep track of the output and the operators in the expression. It also uses a `HashMap` to store the precedence of each operator. The function then iterates through each token in the input vector and performs different actions based on the type of the token.

### `parser::parse_subtract_sign`

This function takes a mutable reference to a vector of `Token`s and parses whether the token `-` is a negative sign or a minus operator. It does this by iterating through each token in the vector and checking if the token is `-`. If the token is `-`, the function checks whether it is a negative sign or a minus operator based on the position of the token in the expression. If the token is a negative sign, the function replaces it with a `Token::Negative` token in the vector. If the token is a minus operator, the function does not modify the vector.

### `parser::parse_implicit_multiplication`

This function takes a mutable reference to a vector of `Token`s and parses whether implicit multiplication is needed between two tokens. Implicit multiplication happens when two tokens are adjacent to each other and there is no operator between them. The function uses two constant arrays, `LEFT_SIDE_IMPLICIT` and `RIGHT_SIDE_IMPLICIT`, to determine whether implicit multiplication is needed. If a token in the left side array is followed by a token in the right side array, the function inserts a `Token::Multiply` token between them in the vector.

## Exposed C API Functions

### `parse_exposed`
The `parse_exposed` function takes a null-terminated C string `input` and returns a pointer to an array of null-terminated C strings. The length of the array is stored in the `outlen` variable, which must be passed as a pointer

### `free_string_array`
To free the memory allocated for the array of C strings returned by `parse_exposed`, use the `free_string_array` function, passing in the pointer to the array and the length of the array as arguments.

Example in C:
```c
// Compiled with `gcc file.c -o file -L. -l:libsquiid_parser.so`
#include <stdio.h>
#include <stdlib.h>

// Define function signatures of Rust functions
extern char** parse_exposed(const char* input, int* outlen);
extern void free_string_array(char** ptr, int len);

int main() {
    // create the string to parse
    const char* input = "3+4*5";
    // array length buffer
    int outlen = 0;

    // call Rust function from shared object
    char** result = parse_exposed(input, &outlen);
    if (result == NULL) {
        printf("Parsing failed.\n");
        return 1;
    }

    // print tokens in order
    for (int i = 0; i < outlen; i++) {
        printf("%s\n", result[i]);
    }

    // free the allocated memory
    free_string_array(result, outlen);
    return 0;
}
```

A similar result can be achieved in other languages which allow you to interact with shared object libraries via the C API, such as Python, Go, and many others. 

## Tokens

Each variant of the `Token` enum represents a type of token. Here is an explanation of each variant:

- `Function(&'a str)`: A function name followed by an opening parenthesis.
- `Comma(&'a str)`: A comma.
- `VariableAssign(&'a str)`: A variable identifier.
- `VariableRecal(&'a str)`: A variable identifier preceded by `$`.
- `Constant(&'a str)`: A constant identifier preceded by `#`.
- `ScientificNotation(&'a str)`: A number in scientific notation format.
- `Float(&'a str)`: A floating-point number.
- `Int(&'a str)`: An integer.
- `PrevAns(&'a str)`: The symbol `@` representing the previous answer.
- `LParen(&'a str)`: An opening parenthesis.
- `RParen(&'a str)`: A closing parenthesis.
- `Equal(&'a str)`: The equals symbol.
- `Power(&'a str)`: The power symbol.
- `Multiply(&'a str)`: The multiplication symbol.
- `Divide(&'a str)`: The division symbol.
- `Modulo(&'a str)`: The modulo symbol.
- `Add(&'a str)`: The addition symbol.
- `Subtract(&'a str)`: The subtraction symbol. This can represent either unary or binary subtraction.
- `Negative(&'a str)`: This is not a valid token, but it is used for differentiation between minus and negative later on in parsing.

The `PartialEq` implementation ignores the content of the `Token` enum and only compares the variants.