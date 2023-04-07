use std::collections::HashMap;
use std::fmt::Error;

use std::collections::VecDeque;

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
    pub fn add_item_to_stack(&mut self, item: Bucket) -> Result<(), Error> {
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
            // TODO: actually error out when undefined variable is referenced
            item_string = self
                .variables
                .get(&item_string)
                .unwrap_or(&Bucket::from(0.0))
                .to_string();
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

        Ok(())
    }

    // Get operands from stack as float
    pub fn get_operands_as_f(&mut self, number: i32) -> Result<Vec<f64>, &'static str> {
        // Make sure there are actually enough items on the stack
        if self.stack.len() as i32 >= number {
            // Create vector to store operands
            let mut operands = Vec::new();
            // check that all items are of expected type
            let requested_operands = &self.stack[self.stack.len() - number as usize..];
            for item in requested_operands {
                if item.bucket_type != BucketTypes::Float {
                    return Err("The operation cannot be performed on these operands");
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
            Err("Not enough items on stack for operation")
        }
    }

    pub fn get_operands_as_string(&mut self, number: i32) -> Result<Vec<String>, &'static str> {
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
            Err("Not enough items on stack for operation")
        }
    }

    pub fn get_operands_raw(&mut self, number: i32) -> Result<Vec<Bucket>, &'static str> {
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
            Err("Not enough items on stack for operation")
        }
    }

    // Add
    pub fn add(&mut self) -> Result<(), &'static str> {
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] + operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(())
    }

    // Subtract
    pub fn subtract(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] - operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(())
    }

    // Multiply
    pub fn multiply(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] * operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(())
    }

    // Divide
    pub fn divide(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] / operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(())
    }

    // Power
    pub fn power(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0].powf(operands[1]);
        let _ = self.add_item_to_stack(result.into());
        Ok(())
    }

    // Square root
    pub fn sqrt(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].sqrt().into());
        Ok(())
    }

    // Modulo
    pub fn modulo(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] % operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(())
    }

    // Sine
    pub fn sin(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].sin().into());
        Ok(())
    }

    // Cosine
    pub fn cos(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].cos().into());
        Ok(())
    }

    // Tangent
    pub fn tan(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].tan().into());
        Ok(())
    }

    // Secant
    pub fn sec(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack((1.0 / operands[0].cos()).into());
        Ok(())
    }

    // Cosecant
    pub fn csc(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack((1.0 / operands[0].sin()).into());
        Ok(())
    }

    // Cotangent
    pub fn cot(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack((1.0 / operands[0].tan()).into());
        Ok(())
    }

    // Asin
    pub fn asin(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].asin().into());
        Ok(())
    }

    // Acos
    pub fn acos(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].acos().into());
        Ok(())
    }

    // Atan
    pub fn atan(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].atan().into());
        Ok(())
    }

    // Invert
    pub fn invert(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = operands[0] * -1.0;
        let _ = self.add_item_to_stack(result.into());
        Ok(())
    }

    // Logarithm
    pub fn log(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].log(10.0).into());
        Ok(())
    }

    // Logarithm with custom base
    pub fn logb(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].log(operands[1]).into());
        Ok(())
    }

    // Natural logarihm
    pub fn ln(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].ln().into());
        Ok(())
    }

    // Absolute value
    pub fn abs(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].abs().into());
        Ok(())
    }

    // Equal to
    pub fn eq(&mut self) -> Result<(), &'static str> {
        // Get operands
        // TODO: maybe make this work with strings
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] == operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(())
    }

    // Greater than
    pub fn gt(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] > operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(())
    }

    // Less than
    pub fn lt(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] < operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(())
    }

    // Greater than or equal to
    pub fn gte(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] >= operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(())
    }

    // Less than or equal to
    pub fn lte(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands_as_f(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = (operands[0] <= operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(())
    }

    // round to nearest int
    pub fn round(&mut self) -> Result<(), &'static str> {
        // Get operand
        let operands = match self.get_operands_as_f(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].round().into());
        Ok(())
    }

    // Drop last item from stack
    pub fn drop(&mut self) -> Result<(), &'static str> {
        // Remove last item from stack
        self.stack.pop();
        Ok(())
    }

    // Swap last two items on stack
    pub fn swap(&mut self) -> Result<(), &'static str> {
        // Get last two values from stack
        let operands = match self.get_operands_raw(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Insert in reverse order
        let _ = self.add_item_to_stack(operands[1].clone());
        let _ = self.add_item_to_stack(operands[0].clone());
        Ok(())
    }

    // Duplicate the last item of the stack
    pub fn dup(&mut self) -> Result<(), &'static str> {
        // Get the last value from the stack
        let operands = match self.get_operands_raw(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Insert twice
        let _ = self.add_item_to_stack(operands[0].clone());
        let _ = self.add_item_to_stack(operands[0].clone());
        Ok(())
    }

    // Roll down
    pub fn roll_down(&mut self) -> Result<(), &'static str> {
        if self.stack.len() > 0 {
            // Rotate stack right
            self.stack.rotate_right(1);
            Ok(())
        } else {
            Err("Cannot roll empty stack")
        }
    }

    // Roll up
    pub fn roll_up(&mut self) -> Result<(), &'static str> {
        if self.stack.len() > 0 {
            // Rotate stack left
            self.stack.rotate_left(1);
            Ok(())
        } else {
            Err("Cannot roll empty stack")
        }
    }

    // Store value in variable
    pub fn store(&mut self) -> Result<(), &'static str> {
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
            return Err("Cannot store in non-variable object");
        }
        Ok(())
    }

    // Delete variable
    pub fn purge(&mut self) -> Result<(), &'static str> {
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
                return Err("Variable does not exist")
            }
        } else {
            // Error if attempted to store in name not starting with @
            return Err("Cannot delete non-variable object");
        }
        Ok(())
    }

    // Store value in variable, with inverted argument order
    pub fn invstore(&mut self) -> Result<(), &'static str> {
        match self.swap() {
            Ok(()) => {},
            Err(error) => return Err(error),
        }
        self.store()
    }

    pub fn clear(&mut self) -> Result<(), &'static str> {
        self.stack = Vec::new();
        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), &'static str> {
        if self.history.len() > 1 {
            _ = self.history.pop_back().unwrap();
            self.stack = self.history.pop_back().unwrap();
            Ok(())
        } else {
            Err("Cannot undo further")
        }
    }
}

// TODO: Write engine tests
