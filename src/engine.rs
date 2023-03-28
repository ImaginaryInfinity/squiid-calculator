use std::collections::HashMap;

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
    pub fn add_item_to_stack(&mut self, item: StackableItems) {
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
            item_string = self.variables[&item_string].to_string();
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
            self.invert();
        }
    }

    // Get operands from stack
    pub fn get_operands(&mut self, number: i32) -> Vec<StackableItems> {
        // Create vector to store operands
        let mut operands = Vec::new();
        // Add requested number of operands from stack to vector and converts them to strings
        for _ in 0..number {
            let operand = self.stack.pop().unwrap();
            operands.push(operand);
        }
        // Make the new vector's order match the stack
        operands.reverse();
        operands
    }

    // Add
    pub fn add(&mut self) {
        let operands = self.get_operands(2);

        // Put result on stack
        self.add_item_to_stack(operands[0].add(operands[1].clone()));
    }

    // Subtract
    pub fn subtract(&mut self) {
        // Get operands
        let operands = self.get_operands(2);

        // Put result on stack
        self.add_item_to_stack(operands[0].sub(operands[1].clone()));
    }

    // Multiply
    pub fn multiply(&mut self) {
        // Get operands
        let operands = self.get_operands(2);

        // Put result on stack
        self.add_item_to_stack(operands[0].mul(operands[1].clone()));
    }

    // Divide
    pub fn divide(&mut self) {
        // Get operands
        let operands = self.get_operands(2);

        // Put result on stack
        self.add_item_to_stack(operands[0].div(operands[1].clone()));
    }

    // Power
    pub fn power(&mut self) {
        // Get operands
        let operands = self.get_operands(2);

        // Put result on stack
        self.add_item_to_stack(operands[0].powf(operands[1].clone()));
    }

    // Square root
    pub fn sqrt(&mut self) {
        // Get operands
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].sqrt());
    }

    // Modulo
    pub fn modulo(&mut self) {
        // Get operands
        let operands = self.get_operands(2);

        // Put result on stack
        self.add_item_to_stack(operands[0].modulo(operands[1].clone()));
    }

    // Sine
    pub fn sin(&mut self) {
        // Get operands
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].sin());
    }

    // Cosine
    pub fn cos(&mut self) {
        // Get operands
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].cos());
    }

    // Tangent
    pub fn tan(&mut self) {
        // Get operands
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].tan());
    }

    // Secant
    pub fn sec(&mut self) {
        // Get operands
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].sec());
    }

    // Cosecant
    pub fn csc(&mut self) {
        // Get operands
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].csc());
    }

    // Cotangent
    pub fn cot(&mut self) {
        // Get operands
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].cot());
    }

    // Asin
    pub fn asin(&mut self) {
        // Get operands
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].asin());
    }

    // Acos
    pub fn acos(&mut self) {
        // Get operands
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].acos());
    }

    // Atan
    pub fn atan(&mut self) {
        // Get operands
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].atan());
    }

    // Invert
    pub fn invert(&mut self) {
        // Get operands
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].mul(StackableFloat(-1.0)));
    }

    // Logarithm
    pub fn log(&mut self) {
        // Get operands
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].log(StackableFloat(10.0)));
    }

    // Logarithm with custom base
    pub fn logb(&mut self) {
        // Get operands
        let operands = self.get_operands(2);

        // Put result on stack
        self.add_item_to_stack(operands[0].log(operands[1].clone()));
    }

    // Natural logarihm
    pub fn ln(&mut self) {
        // Get operands
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].ln());
    }

    // Absolute value
    pub fn abs(&mut self) {
        // Get operands
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].abs());
    }

    // Equal to
    pub fn eq(&mut self) {
        // Get operands
        let operands = self.get_operands(2);

        // Put result on stack
        self.add_item_to_stack(operands[0].eq(operands[1].clone()));
    }

    // Greater than
    pub fn gt(&mut self) {
        // Get operands
        let operands = self.get_operands(2);

        // Put result on stack
        self.add_item_to_stack(operands[0].gt(operands[1].clone()));
    }

    // Less than
    pub fn lt(&mut self) {
        // Get operands
        let operands = self.get_operands(2);

        // Put result on stack
        self.add_item_to_stack(operands[0].lt(operands[1].clone()));
    }

    // Greater than or equal to
    pub fn gte(&mut self) {
        // Get operands
        let operands = self.get_operands(2);

        // Put result on stack
        self.add_item_to_stack(operands[0].gte(operands[1].clone()))
    }

    // Less than or equal to
    pub fn lte(&mut self) {
        // Get operands
        let operands = self.get_operands(2);

        // Put result on stack
        self.add_item_to_stack(operands[0].lte(operands[1].clone()));
    }

    // round to nearest int
    pub fn round(&mut self) {
        // Get operand
        let operands = self.get_operands(1);

        // Put result on stack
        self.add_item_to_stack(operands[0].round());
    }

    // Drop last item from stack
    pub fn drop(&mut self) {
        // Remove last item from stack
        self.stack.pop();
    }

    // Swap last two items on stack
    pub fn swap(&mut self) {
        // Get last two values from stack
        let operands = self.get_operands(2);

        // Insert in reverse order
        self.add_item_to_stack(operands[1].clone());
        self.add_item_to_stack(operands[0].clone());
    }

    // Duplicate the last item of the stack
    pub fn dup(&mut self) {
        // Get the last value from the stack
        let operands = self.get_operands(1);

        // Insert twice
        self.add_item_to_stack(operands[0].clone());
        self.add_item_to_stack(operands[0].clone());
    }

    // Roll down
    pub fn roll_down(&mut self) {
        // Rotate stack right
        self.stack.rotate_right(1);
    }

    // Roll up
    pub fn roll_up(&mut self) {
        // Rotate stack left
        self.stack.rotate_left(1);
    }

    // Store value in variable
    pub fn store(&mut self) {
        // Get 2 operands from stack
        let operands = self.get_operands(2);

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
    }

    pub fn clear(&mut self) {
        self.stack = Vec::new();
    }
}

// TODO: Write engine tests
