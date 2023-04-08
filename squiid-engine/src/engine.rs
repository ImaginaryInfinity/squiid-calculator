use std::collections::HashMap;

use std::collections::VecDeque;

use crate::ResponseType;
use crate::bucket::{Bucket, BucketTypes};
use crate::utils::is_string_numeric;

// Evaluation engine struct
pub struct Engine {
    pub stack: Vec<Bucket>,
    pub variables: HashMap<String, Bucket>,
    pub history: VecDeque<Vec<Bucket>>,
}

// Evaluation engine implementation
impl Engine {
    // helper to construct a new engine object
    pub fn new() -> Engine {
        Engine {
            stack: Vec::new(),
            variables: HashMap::new(),
            history: VecDeque::new(),
        }
    }

    // add item to stack
    pub fn add_item_to_stack(&mut self, item: Bucket) -> Result<ResponseType, String> {
        // Convert item to string
        let mut item_string = item.to_string();
        let mut invert = false;

        // Set to invert at end if beginning with _
        if item_string.chars().next().unwrap() == '_' {
            item_string.remove(0);
            invert = true;
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
            let unresolved_var = self
                .variables
                .get(&item_string);
            
            match unresolved_var {
                Some(value) => item_string = value.to_string(),
                None => return Err(format!("reference to undefined variable: {}", item_string)),
            }
        }

        // create a StackableFloat if item_string is numeric, else StackableString
        let item_pushable: Bucket;
        if is_string_numeric(&item_string) {
            item_pushable = Bucket::from(item_string.parse::<f64>().unwrap());
        } else {
            item_pushable = Bucket::from(item_string);
        }

        // push the new item to the stack
        self.stack.push(item_pushable);

        // Invert if originally began with _
        if invert {
            _ = self.invert();
        }

        Ok(ResponseType::SendStack)
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
                    return Err(String::from("The operation cannot be performed on these operands"));
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
    pub fn add(&mut self) -> Result<ResponseType, String> {
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] + operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(ResponseType::SendStack)
    }

    // Subtract
    pub fn subtract(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] - operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(ResponseType::SendStack)
    }

    // Multiply
    pub fn multiply(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] * operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(ResponseType::SendStack)
    }

    // Divide
    pub fn divide(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] / operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(ResponseType::SendStack)
    }

    // Power
    pub fn power(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0].powf(operands[1]);
        let _ = self.add_item_to_stack(result.into());
        Ok(ResponseType::SendStack)
    }

    // Square root
    pub fn sqrt(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].sqrt().into());
        Ok(ResponseType::SendStack)
    }

    // Modulo
    pub fn modulo(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] % operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(ResponseType::SendStack)
    }

    // Sine
    pub fn sin(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].sin().into());
        Ok(ResponseType::SendStack)
    }

    // Cosine
    pub fn cos(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].cos().into());
        Ok(ResponseType::SendStack)
    }

    // Tangent
    pub fn tan(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].tan().into());
        Ok(ResponseType::SendStack)
    }

    // Secant
    pub fn sec(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack((1.0 / operands[0].cos()).into());
        Ok(ResponseType::SendStack)
    }

    // Cosecant
    pub fn csc(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack((1.0 / operands[0].sin()).into());
        Ok(ResponseType::SendStack)
    }

    // Cotangent
    pub fn cot(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack((1.0 / operands[0].tan()).into());
        Ok(ResponseType::SendStack)
    }

    // Asin
    pub fn asin(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].asin().into());
        Ok(ResponseType::SendStack)
    }

    // Acos
    pub fn acos(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].acos().into());
        Ok(ResponseType::SendStack)
    }

    // Atan
    pub fn atan(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].atan().into());
        Ok(ResponseType::SendStack)
    }

    // Invert
    pub fn invert(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] * -1.0;
        let _ = self.add_item_to_stack(result.into());
        Ok(ResponseType::SendStack)
    }

    // Logarithm
    pub fn log(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].log(10.0).into());
        Ok(ResponseType::SendStack)
    }

    // Logarithm with custom base
    pub fn logb(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].log(operands[1]).into());
        Ok(ResponseType::SendStack)
    }

    // Natural logarihm
    pub fn ln(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].ln().into());
        Ok(ResponseType::SendStack)
    }

    // Absolute value
    pub fn abs(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].abs().into());
        Ok(ResponseType::SendStack)
    }

    // Equal to
    pub fn eq(&mut self) -> Result<ResponseType, String> {
        // Get operands
        // TODO: maybe make this work with strings
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] == operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(ResponseType::SendStack)
    }

    // Greater than
    pub fn gt(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] > operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(ResponseType::SendStack)
    }

    // Less than
    pub fn lt(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] < operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(ResponseType::SendStack)
    }

    // Greater than or equal to
    pub fn gte(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] >= operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(ResponseType::SendStack)
    }

    // Less than or equal to
    pub fn lte(&mut self) -> Result<ResponseType, String> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] <= operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(ResponseType::SendStack)
    }

    // round to nearest int
    pub fn round(&mut self) -> Result<ResponseType, String> {
        // Get operand
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].round().into());
        Ok(ResponseType::SendStack)
    }

    // Drop last item from stack
    pub fn drop(&mut self) -> Result<ResponseType, String> {
        // Remove last item from stack
        self.stack.pop();
        Ok(ResponseType::SendStack)
    }

    // Swap last two items on stack
    pub fn swap(&mut self) -> Result<ResponseType, String> {
        // Get last two values from stack
        let operands = match self.get_operands_raw(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Insert in reverse order
        let _ = self.add_item_to_stack(operands[1].clone());
        let _ = self.add_item_to_stack(operands[0].clone());
        Ok(ResponseType::SendStack)
    }

    // Duplicate the last item of the stack
    pub fn dup(&mut self) -> Result<ResponseType, String> {
        // Get the last value from the stack
        let operands = match self.get_operands_raw(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Insert twice
        let _ = self.add_item_to_stack(operands[0].clone());
        let _ = self.add_item_to_stack(operands[0].clone());
        Ok(ResponseType::SendStack)
    }

    // Roll down
    pub fn roll_down(&mut self) -> Result<ResponseType, String> {
        if self.stack.len() > 0 {
            // Rotate stack right
            self.stack.rotate_right(1);
            Ok(ResponseType::SendStack)
        } else {
            Err(String::from("Cannot roll empty stack"))
        }
    }

    // Roll up
    pub fn roll_up(&mut self) -> Result<ResponseType, String> {
        if self.stack.len() > 0 {
            // Rotate stack left
            self.stack.rotate_left(1);
            Ok(ResponseType::SendStack)
        } else {
            Err(String::from("Cannot roll empty stack"))
        }
    }

    // Store value in variable
    pub fn store(&mut self) -> Result<ResponseType, String> {
        // Get 2 operands from stack
        let operands = match self.get_operands_raw(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Only store if first character of second operand is @
        if operands[1].to_string().chars().next().unwrap() == '@' {
            // Convert name to string
            let mut varname = operands[1].to_string();
            // Remove @ prefix
            varname.remove(0);
            // Add variable to hashmap
            self.variables.insert(varname, operands[0].clone());
        } else {
            // Error if attempted to store in name not starting with @
            return Err(String::from("Cannot store in non-variable object"));
        }
        Ok(ResponseType::SendStack)
    }

    // Delete variable
    pub fn purge(&mut self) -> Result<ResponseType, String> {
        // Get 2 operands from stack
        let operands = match self.get_operands_raw(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Only try to delete if first character of second operand is @
        if operands[0].to_string().chars().next().unwrap() == '@' {
            // Convert name to string
            let mut varname = operands[0].to_string();
            // Remove @ prefix
            varname.remove(0);
            if self.variables.contains_key(&varname) {
                // Remove variable from hashmap
                self.variables.remove(&varname);
            } else {
                return Err(String::from("Variable does not exist"))
            }
        } else {
            // Error if attempted to store in name not starting with @
            return Err(String::from("Cannot delete non-variable object"));
        }
        Ok(ResponseType::SendStack)
    }

    // Store value in variable, with inverted argument order
    pub fn invstore(&mut self) -> Result<ResponseType, String> {
        match self.swap() {
            Ok(_) => {},
            Err(error) => return Err(error),
        }
        self.store()
    }

    pub fn clear(&mut self) -> Result<ResponseType, String> {
        self.stack = Vec::new();
        Ok(ResponseType::SendStack)
    }

    pub fn undo(&mut self) -> Result<ResponseType, String> {
        if self.history.len() > 1 {
            _ = self.history.pop_back().unwrap();
            self.stack = self.history.pop_back().unwrap();
            Ok(ResponseType::SendStack)
        } else {
            Err(String::from("Cannot undo further"))
        }
    }

    pub fn list_commands(&mut self) -> Result<ResponseType, String> {
        Ok(ResponseType::SendCommands)
    }
}

// TODO: Write engine tests
