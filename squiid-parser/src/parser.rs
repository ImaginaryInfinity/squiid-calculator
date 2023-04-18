use std::collections::HashMap;

use log::debug;

use crate::tokens::Token::{self, *};

const LEFT_SIDE_IMPLICIT: [Token; 7] = [
    VariableRecal("_"),
    Constant("_"),
    ScientificNotation("_"),
    Float("_"),
    Int("_"),
    PrevAns("_"),
    RParen("_"),
];

const RIGHT_SIDE_IMPLICIT: [Token; 8] = [
    Function("_"),
    VariableRecal("_"),
    Constant("_"),
    ScientificNotation("_"),
    Float("_"),
    Int("_"),
    PrevAns("_"),
    LParen("_"),
];

pub fn parse_subtract_sign(tokens: &mut Vec<Token>) {
    // parse whether this is a negative sign or a minus operator
    // it is a negative sign if:
    //
    // at the beginning of an expression
    // at the beginning of an opening parenthesis (-3+6)
    // at the beginning of a function (func(-5))
    // after another operator (3+-5, 3*-5, 3^-5)
    // as an argument in a function, so after a comma (function(3, -3))

    let mut negative_replacements: Vec<usize> = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if *token == Subtract("-") {
            // at the beginning of an expression
            if index == 0 {
                negative_replacements.push(index);
            } else {
                // get the token before the current negative sign
                match tokens[index - 1] {
                    // at the beginning of an opening parenthesis (-3+6)
                    Token::LParen("(") |
                    // at the beginning of a function (func(-5))
                    Token::Function(_) |
                    // after another operator (3+-5, 3*-5, 3^-5)
                    Token::Add("+") | Token::Subtract("-") | Token::Modulo("%") | Token::Multiply("*") | Token::Divide("/") | Token::Power("^") | Token::Equal("=") |
                    // as an argument in a function, so after a comma (function(3, -3))
                    Token::Comma(",") => {
                        negative_replacements.push(index);
                    },
                    _ => (),
                }
            }
        }
    }

    // do replacements
    for index in negative_replacements {
        tokens[index] = Negative("-");
    }
}

pub fn parse_implicit_multiplication(tokens: &mut Vec<Token>) {
    // Left side (current token):
    // Function, VariableRecal, Constant, ScientificNotation, Float, Int, PrevAns, RParen
    //
    // Right Side (peek token):
    // Function, VariableRecal, Constant, ScientificNotation, Float, Int, PrevAns, LParen
    //
    // Implicit multiplication happens if something on the left side list is followed by something on the right side list

    let mut multiply_insertions = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if LEFT_SIDE_IMPLICIT.contains(&token) {
            // make sure that we aren't on the last token
            if index + 1 != tokens.len() {
                // there is a next token that is not an error, test if in right side multiplication table
                if RIGHT_SIDE_IMPLICIT.contains(&tokens[index + 1]) {
                    // implicit multiplication is needed at the index in front of the current token
                    multiply_insertions.push(index + 1);
                }
            };
        }
    }

    // do insertions
    for (index, insertion_index) in multiply_insertions.iter().enumerate() {
        // offset by index to take account for tokens vec growing in size
        tokens.insert(*insertion_index + index, Multiply("*"));
    }
}

pub fn shunting_yard_parser<'a>(tokens: Vec<Token<'a>>) -> Result<Vec<&'a str>, String> {
    debug!("{:?}", tokens);

    let mut output_queue: Vec<&'a str> = Vec::new();
    let mut operator_stack: Vec<&'a str> = Vec::new();

    // keep track of when we need to insert a chs function
    // this is used for parenthesis (insert at end of expression)
    let mut chs_parenthesis: Vec<u32> = Vec::new();
    // this is used for all other tokens (insert directly after)
    let mut insert_chs = false;

    let precedence_map = HashMap::from([
        ("^", 4),
        ("*", 3),
        ("/", 3),
        ("%", 3),
        ("+", 2),
        ("-", 2),
        ("(", 1),
    ]);

    let mut peekable_tokens = tokens.iter().peekable();

    while let Some(token) = peekable_tokens.next() {
        debug!(
            "output: {:?}, operator: {:?}, chs_function: {:?}",
            output_queue, operator_stack, chs_parenthesis
        );
        match token {
            Token::Function(token_name) => {
                operator_stack.push(token_name.trim_end_matches('('));
                operator_stack.push("(");

                // increment every element of insert_chs_function
                for num in &mut chs_parenthesis {
                    *num += 1;
                }
            }
            Token::LParen(token_name) => {
                operator_stack.push(token_name);

                // increment every element of insert_chs_function
                for num in &mut chs_parenthesis {
                    *num += 1;
                }
            }
            Token::VariableAssign(token_name)
            | Token::VariableRecal(token_name)
            | Token::Constant(token_name)
            | Token::ScientificNotation(token_name)
            | Token::Float(token_name)
            | Token::Int(token_name)
            | Token::PrevAns(token_name) => {
                output_queue.push(token_name);
            }
            Token::Comma(_) => {
                while let Some(operator) = operator_stack.pop() {
                    if operator == "(" {
                        operator_stack.push(operator);
                        break;
                    } else {
                        output_queue.push(operator);
                    }
                }
            }
            Token::RParen(_) => {
                // decrement every element of insert_chs_function
                for num in &mut chs_parenthesis {
                    *num -= 1;
                }

                while let Some(operator) = operator_stack.pop() {
                    if operator == "(" {
                        break;
                    } else {
                        output_queue.push(operator);
                    }
                }

                // if there is an element which is 0, we are at the end of the
                // function or parenthesis and we need to apply the chs function
                for i in 0..chs_parenthesis.len() {
                    if chs_parenthesis[i] == 0 {
                        // insert the chs function second from top on the operator stack
                        let insertion_index = if operator_stack.len() < 2 {
                            0
                        } else {
                            operator_stack.len() - 2
                        };

                        operator_stack.insert(insertion_index, "chs");

                        // remove the element in the list which is 0
                        chs_parenthesis.remove(i);
                        break;
                    }
                }
            }
            Token::Equal(token_name)
            | Token::Power(token_name)
            | Token::Multiply(token_name)
            | Token::Divide(token_name)
            | Token::Modulo(token_name)
            | Token::Add(token_name)
            | Token::Subtract(token_name) => {
                while let Some(operator) = operator_stack.pop() {
                    if operator == "("
                        // functions should have the highest precedence  
                        || precedence_map.get(operator).unwrap_or(&100) < precedence_map.get(token_name).unwrap_or(&0)
                    {
                        operator_stack.push(operator);
                        break;
                    } else {
                        output_queue.push(operator);
                    }
                }
                operator_stack.push(token_name);
            }
            Token::Negative(_) => match peekable_tokens.peek() {
                Some(next_token) => match next_token {
                    Token::Function(_) | Token::LParen(_) => {
                        chs_parenthesis.push(0);
                    }
                    _ => {
                        // insert the chs function after the next token
                        insert_chs = true;
                        continue;
                    }
                },
                None => {
                    return Err("Trailing negative sign".to_string());
                }
            },
        }

        if insert_chs {
            output_queue.push("chs");
            insert_chs = false;
        }
    }

    while let Some(operator) = operator_stack.pop() {
        output_queue.push(operator);
    }

    Ok(output_queue)
}
