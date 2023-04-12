use std::collections::HashMap;
use log::debug;

pub mod exposed;

// Left/Right Associativity enum for distinguising between right associative operations such as power
#[derive(PartialEq)]
enum Associativity {
    Left,
    Right,
}

// operator properites class that contains the operator precendence and the associativity
struct OperatorProperties {
    precedence: u8,
    associativity: Associativity,
}

// function to check if a string is numeric. _ indicates negative numbers
fn is_string_numeric(str: &str) -> bool {
    for c in str.chars() {
        // return false if the current character is not numeric or is not a valid numerical character, such as . for decimals and _ for indicative negatives
        if !c.is_numeric()
            && !(['.', '-', 'e'].contains(&c)
                && str.chars().count() > 1
                && !['-', 'e'].contains(&(str.chars().last().unwrap())))
        {
            return false;
        }
    }
    return true;
}

// function to check if a string is alphabetic
fn is_string_alphabetic(str: &str) -> bool {
    for c in str.chars() {
        if !c.is_alphabetic() {
            return false;
        }
    }
    return true;
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classifier_functions() {
        // test numeric function
        assert_eq!(is_string_numeric("abc"), false);
        assert_eq!(is_string_numeric("1a"), false);
        assert_eq!(is_string_numeric("12e.a"), false);
        assert_eq!(is_string_numeric("123"), true);
        assert_eq!(is_string_numeric("1.2"), true);
        assert_eq!(is_string_numeric("1.2e7"), true);

        // test alphabetic function
        assert_eq!(is_string_alphabetic("abc"), true);
        assert_eq!(is_string_alphabetic("1a"), false);
        assert_eq!(is_string_alphabetic("12e.a"), false);
        assert_eq!(is_string_alphabetic("123"), false);
        assert_eq!(is_string_alphabetic("1.2"), false);
        assert_eq!(is_string_alphabetic("1.2e7"), false);
    }

    #[test]
    fn test_parsing() {
        assert_eq!(
            parse("sqrt(5*(((((1+0.2*(350/661.5)^2)^3.5-1)*(1-(6.875*10^_6)*25500)^_5.2656)+1)^0.286-1))"),
            vec!["5", "1", "0.2", "350", "661.5", "/", "2", "^", "*", "+", "3.5", "^", "1", "-", "1", "6.875", "10", "_6", "^", "*", "25500", "*", "-", "_5.2656", "^", "*", "1", "+", "0.286", "^", "1", "-", "*", "sqrt"]
        );

        assert_eq!(parse("$A * $B + $C"), vec!["$A", "$B", "*", "$C", "+"]);

        assert_eq!(parse("$A + $B * $C"), vec!["$A", "$B", "$C", "*", "+"]);

        assert_eq!(parse("$A * ($B + $C)"), vec!["$A", "$B", "$C", "+", "*"]);

        assert_eq!(
            parse("34 * 5.3 ^ 2 + 0.9"),
            vec!["34", "5.3", "2", "^", "*", "0.9", "+"]
        );

        assert_eq!(
            parse("8e3 * ($B + 4.532 * _0.2) + $A"),
            vec!["8e3", "$B", "4.532", "_0.2", "*", "+", "*", "$A", "+"]
        );
    }
}
