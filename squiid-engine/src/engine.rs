use std::collections::HashMap;
use std::collections::VecDeque;

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, MathematicalOps};

use crate::bucket::{Bucket, BucketTypes};
use crate::utils::{ID_REGEX, NUMERIC_REGEX};
use crate::MessageAction;

// Evaluation engine struct
pub struct Engine {
    pub stack: Vec<Bucket>,
    pub variables: HashMap<String, Bucket>,
    pub history: VecDeque<Vec<Bucket>>,
    pub variable_history: VecDeque<HashMap<String, Bucket>>,
}

// Evaluation engine implementation
impl Engine {
    // helper to construct a new engine object
    pub fn new() -> Engine {
        Engine {
            stack: Vec::new(),
            variables: HashMap::new(),
            history: VecDeque::new(),
            variable_history: VecDeque::new(),
        }
    }

    // add item to stack
    pub fn add_item_to_stack(&mut self, item: Bucket) -> Result<MessageAction, String> {
        // Convert item to string
        let mut item_string = item.to_string();
        let mut chs = false;

        // Set to change sign at end if beginning with _
        if item_string.chars().next().unwrap() == '_' {
            item_string.remove(0);
            chs = true;
        }

        // Replace with value if item is a constant
        item_string = match item_string.as_str() {
            "#pi" => std::f64::consts::PI.to_string(),
            "#e" => std::f64::consts::E.to_string(),
            "#tau" => std::f64::consts::TAU.to_string(),
            "#c" => "299792458".to_string(),
            "#G" => (6.67430 * 10_f64.powf(-11_f64)).to_string(),
            _ => item_string,
        };

        // Replace with value if item is a variable
        if item_string.chars().next().unwrap() == '$' {
            // Remove $ prefix from name
            item_string.remove(0);
            // Get variable from hashmap
            let unresolved_var = self.variables.get(&item_string);

            match unresolved_var {
                Some(value) => item_string = value.to_string(),
                None => return Err(format!("reference to undefined variable: {}", item_string)),
            }
        }

        // create a StackableFloat if item_string is numeric, else StackableString
        let item_pushable: Bucket;
        eprintln!("{:?}", item_string);
        if NUMERIC_REGEX.is_match(&item_string) {
            item_pushable = Bucket::from(item_string.parse::<f64>().unwrap());
        } else {
            item_pushable = Bucket::from(item_string);
        }

        // push the new item to the stack
        self.stack.push(item_pushable);

        // Change sign if originally began with _
        if chs {
            _ = self.chs();
        }

        Ok(MessageAction::SendStack)
    }

    // Get operands from stack as float
    pub fn get_operands_as_f(&mut self, number: i32) -> Result<Vec<f64>, String> {
        // Make sure there are actually enough items on the stack
        if self.stack.len() as i32 >= number {
            // Create vector to store operands
            let mut operands = Vec::new();
            // check that all items are of expected type
            let requested_operands = &self.stack[self.stack.len() - number as usize..];
            for item in requested_operands {
                if item.bucket_type != BucketTypes::Float {
                    return Err(String::from(
                        "The operation cannot be performed on these operands",
                    ));
                }
            }

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self.stack.pop().unwrap();
                operands.push(operand.value.parse::<f64>().unwrap());
            }
            // Make the new vector's order match the stack
            operands.reverse();
            Ok(operands)
        } else {
            Err(String::from("Not enough items on stack for operation"))
        }
    }

    pub fn get_operands_as_dec(&mut self, number: i32) -> Result<Vec<Decimal>, String> {
        // Make sure there are actually enough items on the stack
        if self.stack.len() as i32 >= number {
            // Create vector to store operands
            let mut operands = Vec::new();
            // check that all items are of expected type
            let requested_operands = &self.stack[self.stack.len() - number as usize..];
            for item in requested_operands {
                if item.bucket_type != BucketTypes::Float {
                    return Err(String::from(
                        "The operation cannot be performed on these operands",
                    ));
                }
            }

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self.stack.pop().unwrap();
                operands.push(Decimal::from_str_exact(&operand.value).unwrap());
            }
            // Make the new vector's order match the stack
            operands.reverse();
            Ok(operands)
        } else {
            Err(String::from("Not enough items on stack for operation"))
        }
    }

    pub fn get_operands_as_string(&mut self, number: i32) -> Result<Vec<String>, String> {
        // Make sure there are actually enough items on the stack
        if self.stack.len() as i32 >= number {
            // Create vector to store operands
            let mut operands = Vec::new();
            // we can skip the type check since everything is already a string

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self.stack.pop().unwrap();

                operands.push(operand.value);
            }
            // Make the new vector's order match the stack
            operands.reverse();
            Ok(operands)
        } else {
            Err(String::from("Not enough items on stack for operation"))
        }
    }

    pub fn get_operands_raw(&mut self, number: i32) -> Result<Vec<Bucket>, String> {
        if self.stack.len() as i32 >= number {
            // Create vector to store operands
            let mut operands = Vec::new();

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self.stack.pop().unwrap();

                operands.push(operand);
            }
            // Make the new vector's order match the stack
            operands.reverse();
            Ok(operands)
        } else {
            Err(String::from("Not enough items on stack for operation"))
        }
    }

    // Add
    pub fn add(&mut self) -> Result<MessageAction, String> {
        let operands = match self.get_operands_as_dec(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] + operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Subtract
    pub fn subtract(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_dec(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] - operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Multiply
    pub fn multiply(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_dec(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] * operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Divide
    pub fn divide(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_dec(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] / operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Power
    pub fn power(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_dec(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        let base = operands[0];
        let exponent = operands[1];

        // TODO: consider adding the option to use both rust_decimal and rug
        // detect if exponent is decimal, if so, don't use decimal library as that estimates
        let result = if exponent.fract() == Decimal::from_f64(0.0).unwrap() {
            // is not a decimal
            match base.checked_powd(exponent) {
                Some(value) => value.to_f64().unwrap(),
                None => return Err("overflow when raising to a power".to_string()),
            }
        } else {
            // is a decimal
            base.to_f64().unwrap().powf(exponent.to_f64().unwrap())
        };

        // Put result on stack
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Square root
    pub fn sqrt(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_dec(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = match operands[0].sqrt() {
            Some(value) => value,
            None => return Err("Error calculating sqrt".to_string()),
        };
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Modulo
    pub fn modulo(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] % operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Sine
    pub fn sin(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].sin().into());
        Ok(MessageAction::SendStack)
    }

    // Cosine
    pub fn cos(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].cos().into());
        Ok(MessageAction::SendStack)
    }

    // Tangent
    pub fn tan(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].tan().into());
        Ok(MessageAction::SendStack)
    }

    // Secant
    pub fn sec(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack((1.0 / operands[0].cos()).into());
        Ok(MessageAction::SendStack)
    }

    // Cosecant
    pub fn csc(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack((1.0 / operands[0].sin()).into());
        Ok(MessageAction::SendStack)
    }

    // Cotangent
    pub fn cot(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack((1.0 / operands[0].tan()).into());
        Ok(MessageAction::SendStack)
    }

    // Asin
    pub fn asin(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].asin().into());
        Ok(MessageAction::SendStack)
    }

    // Acos
    pub fn acos(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].acos().into());
        Ok(MessageAction::SendStack)
    }

    // Atan
    pub fn atan(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].atan().into());
        Ok(MessageAction::SendStack)
    }

    // Change sign
    pub fn chs(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] * -1.0;
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Logarithm
    pub fn log(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_dec(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = match operands[0].checked_log10() {
            Some(value) => value,
            None => return Err("cannot take log10 of 0 or negative numbers".to_string()),
        };
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Logarithm with custom base using the change of base formula
    pub fn logb(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_dec(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // change of base formula is defined as follows:
        // log_b(a) = (log_d(a))/(log_d(b))

        let top_log = match operands[0].checked_log10() {
            Some(value) => value,
            None => return Err("cannot take log of 0 or negative numbers".to_string()),
        };
        let bottom_log = match operands[1].checked_log10() {
            Some(value) => value,
            None => return Err("cannot take log with base of 0 or negative numbers".to_string()),
        };

        let result = top_log / bottom_log;

        // Put result on stack
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Natural logarihm
    pub fn ln(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_dec(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = match operands[0].checked_ln() {
            Some(value) => value,
            None => return Err("cannot take log10 of 0 or negative numbers".to_string()),
        };
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Absolute value
    pub fn abs(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].abs().into());
        Ok(MessageAction::SendStack)
    }

    // Equal to
    pub fn eq(&mut self) -> Result<MessageAction, String> {
        // Get operands
        // TODO: maybe make this work with strings
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] == operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Greater than
    pub fn gt(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] > operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Less than
    pub fn lt(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] < operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Greater than or equal to
    pub fn gte(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] >= operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // Less than or equal to
    pub fn lte(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] <= operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(MessageAction::SendStack)
    }

    // round to nearest int
    pub fn round(&mut self) -> Result<MessageAction, String> {
        // Get operand
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].round().into());
        Ok(MessageAction::SendStack)
    }

    // Calculate 1/x
    pub fn invert(&mut self) -> Result<MessageAction, String> {
        // Get operand
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack((1_f64 / operands[0]).into());
        Ok(MessageAction::SendStack)
    }

    // Drop last item from stack
    pub fn drop(&mut self) -> Result<MessageAction, String> {
        // Remove last item from stack
        self.stack.pop();
        Ok(MessageAction::SendStack)
    }

    // Swap last two items on stack
    pub fn swap(&mut self) -> Result<MessageAction, String> {
        // Get last two values from stack
        let operands = match self.get_operands_raw(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Insert in reverse order
        let _ = self.add_item_to_stack(operands[1].clone());
        let _ = self.add_item_to_stack(operands[0].clone());
        Ok(MessageAction::SendStack)
    }

    // Duplicate the last item of the stack
    pub fn dup(&mut self) -> Result<MessageAction, String> {
        // Get the last value from the stack
        let operands = match self.get_operands_raw(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Insert twice
        let _ = self.add_item_to_stack(operands[0].clone());
        let _ = self.add_item_to_stack(operands[0].clone());
        Ok(MessageAction::SendStack)
    }

    // Roll down
    pub fn roll_down(&mut self) -> Result<MessageAction, String> {
        if self.stack.len() > 0 {
            // Rotate stack right
            self.stack.rotate_right(1);
            Ok(MessageAction::SendStack)
        } else {
            Err(String::from("Cannot roll empty stack"))
        }
    }

    // Roll up
    pub fn roll_up(&mut self) -> Result<MessageAction, String> {
        if self.stack.len() > 0 {
            // Rotate stack left
            self.stack.rotate_left(1);
            Ok(MessageAction::SendStack)
        } else {
            Err(String::from("Cannot roll empty stack"))
        }
    }

    // Store value in variable
    pub fn store(&mut self) -> Result<MessageAction, String> {
        // Get 2 operands from stack
        let operands = match self.get_operands_raw(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Only store if matches the identifier pattern
        let varname = operands[1].to_string();
        if ID_REGEX.is_match(&varname) {
            // Add variable to hashmap
            self.variables.insert(varname, operands[0].clone());
        } else {
            // Error if attempted to store in name which is not a valid ID
            return Err(format!("Cannot store in non-variable object `{}`", varname));
        }
        Ok(MessageAction::SendStack)
    }

    // Delete variable
    pub fn purge(&mut self) -> Result<MessageAction, String> {
        // Get operand from stack
        let operands = match self.get_operands_raw(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        let varname = operands[0].to_string();
        if ID_REGEX.is_match(&varname) {
            if self.variables.contains_key(&varname) {
                // Remove variable from hashmap
                self.variables.remove(&varname);
            } else {
                return Err(format!("Variable `{}` does not exist", varname));
            }
        } else {
            // Error if attempted to purge name which is not a valid ID
            return Err(format!("Cannot delete non-variable object `{}`", varname));
        }
        Ok(MessageAction::SendStack)
    }

    // Store value in variable, with inverted argument order
    pub fn invstore(&mut self) -> Result<MessageAction, String> {
        match self.swap() {
            Ok(_) => {}
            Err(error) => return Err(error),
        }
        self.store()
    }

    pub fn clear(&mut self) -> Result<MessageAction, String> {
        self.stack = Vec::new();
        Ok(MessageAction::SendStack)
    }

    pub fn undo(&mut self) -> Result<MessageAction, String> {
        if self.history.len() > 1 {
            // Throw away current stack
            _ = self.history.pop_back();
            // Restore previous stack
            self.stack = self.history.pop_back().unwrap();
            // Throw away current state of variables
            _ = self.variable_history.pop_back();
            // Restore previous state of variables
            self.variables = self.variable_history.pop_back().unwrap();
            Ok(MessageAction::SendStack)
        } else {
            Err(String::from("Cannot undo further"))
        }
    }

    pub fn list_commands(&mut self) -> Result<MessageAction, String> {
        Ok(MessageAction::SendCommands)
    }
}
