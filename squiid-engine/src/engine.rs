use std::collections::HashMap;
use std::fmt::Error;

use crate::stackable_items::StackableItems::{self, StackableFloat, StackableString};
use crate::utils::is_string_numeric;

// Evaluation engine struct
pub struct Engine {
    pub stack: Vec<StackableItems>,
    pub variables: HashMap<String, StackableItems>,
}

// Evaluation engine implementation
impl Engine {
    // helper to construct a new engine object
    pub fn new() -> Engine {
        Engine {
            stack: Vec::new(),
            variables: HashMap::new(),
        }
    }

    // add item to stack
    pub fn add_item_to_stack(&mut self, item: StackableItems) -> Result<(), Error> {
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
            item_string = self.variables.get(&item_string).unwrap_or(&StackableString(String::from("0"))).to_string();
        }

        // create a StackableFloat if item_string is numeric, else StackableString
        let item_pushable: StackableItems;
        if is_string_numeric(&item_string) {
            item_pushable = StackableFloat(item_string.parse::<f64>().unwrap());
        } else {
            item_pushable = StackableString(item_string);
        }

        // push the new item to the stack
        self.stack.push(item_pushable);

        // Invert if originally began with _
        if invert {
            _ = self.invert();
        }

        Ok(())
    }

    // Get operands from stack
    pub fn get_operands(&mut self, number: i32) -> Result<Vec<StackableItems>, &'static str> {
        // Make sure there are actually enough items on the stack
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
        let operands = match self.get_operands(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].add(operands[1].clone()));
        Ok(())
    }

    // Subtract
    pub fn subtract(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].sub(operands[1].clone()));
        Ok(())
    }

    // Multiply
    pub fn multiply(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].mul(operands[1].clone()));
        Ok(())
    }

    // Divide
    pub fn divide(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].div(operands[1].clone()));
        Ok(())
    }

    // Power
    pub fn power(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].powf(operands[1].clone()));
        Ok(())
    }

    // Square root
    pub fn sqrt(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].sqrt());
        Ok(())
    }

    // Modulo
    pub fn modulo(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].modulo(operands[1].clone()));
        Ok(())
    }

    // Sine
    pub fn sin(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].sin());
        Ok(())
    }

    // Cosine
    pub fn cos(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].cos());
        Ok(())
    }

    // Tangent
    pub fn tan(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].tan());
        Ok(())
    }

    // Secant
    pub fn sec(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].sec());
        Ok(())
    }

    // Cosecant
    pub fn csc(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].csc());
        Ok(())
    }

    // Cotangent
    pub fn cot(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].cot());
        Ok(())
    }

    // Asin
    pub fn asin(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].asin());
        Ok(())
    }

    // Acos
    pub fn acos(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].acos());
        Ok(())
    }

    // Atan
    pub fn atan(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].atan());
        Ok(())
    }

    // Invert
    pub fn invert(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].mul(StackableFloat(-1.0)));
        Ok(())
    }

    // Logarithm
    pub fn log(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].log(StackableFloat(10.0)));
        Ok(())
    }

    // Logarithm with custom base
    pub fn logb(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].log(operands[1].clone()));
        Ok(())
    }

    // Natural logarihm
    pub fn ln(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].ln());
        Ok(())
    }

    // Absolute value
    pub fn abs(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].abs());
        Ok(())
    }

    // Equal to
    pub fn eq(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].eq(operands[1].clone()));
        Ok(())
    }

    // Greater than
    pub fn gt(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].gt(operands[1].clone()));
        Ok(())
    }

    // Less than
    pub fn lt(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].lt(operands[1].clone()));
        Ok(())
    }

    // Greater than or equal to
    pub fn gte(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].gte(operands[1].clone()));
        Ok(())
    }

    // Less than or equal to
    pub fn lte(&mut self) -> Result<(), &'static str> {
        // Get operands
        let operands = match self.get_operands(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].lte(operands[1].clone()));
        Ok(())
    }

    // round to nearest int
    pub fn round(&mut self) -> Result<(), &'static str> {
        // Get operand
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        self.add_item_to_stack(operands[0].round());
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
        let operands = match self.get_operands(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Insert in reverse order
        self.add_item_to_stack(operands[1].clone());
        self.add_item_to_stack(operands[0].clone());
        Ok(())
    }

    // Duplicate the last item of the stack
    pub fn dup(&mut self) -> Result<(), &'static str> {
        // Get the last value from the stack
        let operands = match self.get_operands(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Insert twice
        self.add_item_to_stack(operands[0].clone());
        self.add_item_to_stack(operands[0].clone());
        Ok(())
    }

    // Roll down
    pub fn roll_down(&mut self) -> Result<(), &'static str> {
        // Rotate stack right
        self.stack.rotate_right(1);
        Ok(())
    }

    // Roll up
    pub fn roll_up(&mut self) -> Result<(), &'static str> {
        // Rotate stack left
        self.stack.rotate_left(1);
        Ok(())
    }

    // Store value in variable
    pub fn store(&mut self) -> Result<(), &'static str> {
        // Get 2 operands from stack
        let operands = match self.get_operands(2) {
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
            panic!("Cannot store in non-variable object");
        }
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), &'static str> {
        self.stack = Vec::new();
        Ok(())
    }
}

// TODO: Write engine tests
