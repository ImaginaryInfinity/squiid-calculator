use std::collections::HashMap;

use log::debug;

use crate::tokens::Token;

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

    let operator_mappings = HashMap::from([
        ("+", "add"),
        ("-", "subtract"),
        ("/", "divide"),
        ("*", "multiply"),
        ("%", "mod"),
        ("^", "power"),
        ("=", "invstore"),
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
                        output_queue.push(operator_mappings.get(operator).unwrap_or(&operator));
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
                        output_queue.push(operator_mappings.get(operator).unwrap_or(&operator));
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
                        || precedence_map.get(operator) < precedence_map.get(token_name)
                    {
                        operator_stack.push(operator);
                        break;
                    } else {
                        output_queue.push(operator_mappings.get(operator).unwrap_or(&operator));
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
        output_queue.push(operator_mappings.get(operator).unwrap_or(&operator));
    }

    Ok(output_queue)
}
