use std::collections::HashMap;

use log::debug;

use crate::tokens::Token;

pub fn shunting_yard_parser<'a>(tokens: Vec<Token<'a>>) -> Result<Vec<&'a str>, String> {
    debug!("{:?}", tokens);

    let mut output_queue: Vec<&'a str> = Vec::new();
    let mut operator_stack: Vec<&'a str> = Vec::new();
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

    for token in tokens {
        debug!("output: {:?}, operator: {:?}", output_queue, operator_stack);
        match token {
            Token::Function(token_name) => {
                operator_stack.push(token_name.trim_end_matches('('));
                operator_stack.push("(");
            }
            Token::LParen(token_name) => {
                operator_stack.push(token_name);
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
                while let Some(operator) = operator_stack.pop() {
                    if operator == "(" {
                        break;
                    } else {
                        output_queue.push(operator_mappings.get(operator).unwrap_or(&operator));
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
        }
    }

    while let Some(operator) = operator_stack.pop() {
        output_queue.push(operator_mappings.get(operator).unwrap_or(&operator));
    }

    Ok(output_queue)
}
