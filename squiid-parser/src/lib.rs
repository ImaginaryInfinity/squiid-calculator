use log::debug;
use std::collections::HashMap;
use utils::{
    is_string_alphabetic,
    is_string_numeric,
    Associativity,
    OperatorProperties
};

pub mod utils;

// main shunting-yard parsing function
pub fn parse(input: &str) -> Vec<String> {
    // construct a hashmap of each valid operator and it's associated properties. a higher precedence signifies a higher priority in order of operations
    let operators = HashMap::from([
        (
            "=",
            OperatorProperties {
                precedence: 5,
                associativity: Associativity::Left,
            },
        ),
        (
            "^",
            OperatorProperties {
                precedence: 4,
                associativity: Associativity::Right,
            },
        ),
        (
            "*",
            OperatorProperties {
                precedence: 3,
                associativity: Associativity::Left,
            },
        ),
        (
            "/",
            OperatorProperties {
                precedence: 3,
                associativity: Associativity::Left,
            },
        ),
        (
            "%",
            OperatorProperties {
                precedence: 3,
                associativity: Associativity::Left,
            },
        ),
        (
            "+",
            OperatorProperties {
                precedence: 2,
                associativity: Associativity::Left,
            },
        ),
        (
            "-",
            OperatorProperties {
                precedence: 2,
                associativity: Associativity::Left,
            },
        ),
    ]);

    // construct stacks
    let mut output_stack: Vec<String> = Vec::new();
    let mut operator_stack: Vec<String> = Vec::new();
    // current token being consumed. may be a variable, function name, or number
    let mut current_token = String::new();

    // iterate over each character in the provided algebraic string
    for char in input.chars() {
        if char.is_numeric() || ['.', '_', '$', '@', 'e'].contains(&char) || char.is_alphabetic() {
            // is part of a number, push to number buffer
            current_token.push(char);
        } else {
            debug!("output: {:?}, operator: {:?}", output_stack, operator_stack);
            // not a part of a number

            // test if current token is not empty, if so, push it to output stack
            if !current_token.is_empty() {
                if is_string_numeric(&current_token)
                    // variables
                    || current_token.starts_with('$')
                    // constants
                    || current_token.starts_with('#')
                    // assigning variables
                    || current_token.starts_with('@')
                {
                    // numbers or variables go on the output stack
                    output_stack.push(current_token.clone());
                } else {
                    // functions go on the operator stack
                    operator_stack.push(current_token.clone());
                }

                // reset the current tokens
                current_token = String::new();
            }

            // check for closing parenthesis
            if char == ')' {
                // pop everything in front of the opening parenthesis off of the operator stack and into the output stack
                while !operator_stack.is_empty() && operator_stack.last().unwrap() != "(" {
                    let operator = operator_stack.pop();
                    match operator {
                        Some(value) => output_stack.push(value),
                        None => panic!("Error: mismatched parenthesis"),
                    }
                }

                // discard opening parenthesis
                operator_stack.pop();

            // push opening parenthesis to stack
            } else if char == '(' {
                operator_stack.push(char.to_string());

            // match any of the defined operators
            } else if operators.keys().into_iter().any(|&i| i == char.to_string()) {
                // check operator precedence
                // the highest thing on the operator stacked will be popped onto the output stack if:
                //
                // the operator stack is not empty AND the top item in the operator stack is not an opening parenthesis
                // OR
                // the current operator's precedence is less than or equal to the precedence of the operator on the top of the stack AND the current operator's associativity is left
                //
                // if neither of these are true, the operator will just be pushed to the operator stack
                while (!operator_stack.is_empty())
                    && (operator_stack.last().unwrap() != "(")
                    && (is_string_alphabetic(operator_stack.last().unwrap())
                        || ((operators[&char.to_string().as_str()].precedence
                            <= operators[&operator_stack.last().unwrap().as_str()].precedence)
                            && operators[&char.to_string().as_str()].associativity
                                == Associativity::Left))
                {
                    // operator precedence is higher or equal to the top one on the stack, pop the stack
                    output_stack.push(operator_stack.pop().unwrap().clone());
                }

                // add the current char to the operator stack
                operator_stack.push(char.to_string());
            }
        }
    }

    // push the number buffer if not empty
    if !current_token.is_empty() {
        output_stack.push(current_token);
    }

    // pop the remainder of the operator stack to the output stack
    while !operator_stack.is_empty() {
        output_stack.push(operator_stack.pop().unwrap().clone());
    }

    output_stack
}
