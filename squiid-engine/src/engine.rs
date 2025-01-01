use std::collections::{HashMap, HashSet, VecDeque};

use rust_decimal::{prelude::ToPrimitive, Decimal, MathematicalOps};
use rust_decimal_macros::dec;

use crate::{
    bucket::{Bucket, BucketTypes, ConstantTypes, CONSTANT_IDENTIFIERS},
    utils::{ID_REGEX, NUMERIC_REGEX},
    EngineSignal,
};

/// Evaluation engine struct
pub struct Engine {
    /// The stack of bucket items
    pub stack: Vec<Bucket>,
    /// Hashmap of set variables
    pub variables: HashMap<String, Bucket>,
    /// History vecdeque for undo support
    pub undo_history: VecDeque<Vec<Bucket>>,
    /// Variables vecdeque for undo support
    pub undo_variable_history: VecDeque<HashMap<String, Bucket>>,
    /// Offset pointer to the current index of the undo history.
    /// Index will be calculated by history.len() - pointer - 1
    pub undo_state_pointer: u8,
    /// Previous answer
    pub previous_answer: Bucket,
}

/// Evaluation engine implementation
impl Engine {
    /// Helper to construct a new engine object
    pub fn new() -> Engine {
        Engine {
            stack: Vec::new(),
            variables: HashMap::new(),
            undo_history: VecDeque::new(),
            undo_variable_history: VecDeque::new(),
            undo_state_pointer: 0,
            previous_answer: Bucket::from(0),
        }
    }

    /// Add item to stack
    pub fn add_item_to_stack(&mut self, item: Bucket) -> Result<EngineSignal, String> {
        // Convert item to string
        let mut item_string = item.to_string();

        // substitute previous answer
        if item_string == "@" {
            item_string = self.previous_answer.to_string();
        }

        // Replace with value if item is a variable
        if item_string.starts_with('$') {
            // Remove $ prefix from name
            item_string.remove(0);
            // Get variable from hashmap
            let unresolved_var = self.variables.get(&item_string);

            match unresolved_var {
                Some(value) => item_string = value.to_string(),
                None => return Err(format!("reference to undefined variable: {}", item_string)),
            }
        }

        // create a Float if item_string is numeric, else String
        let item_pushable: Bucket = match item.bucket_type {
            BucketTypes::Undefined => Bucket::new_undefined(),
            BucketTypes::Constant(constant_type) => {
                // bucket already has a constant type, use that
                Bucket::from_constant(constant_type)
            }
            BucketTypes::Float | BucketTypes::String => {
                // test all other options
                if CONSTANT_IDENTIFIERS.contains_key(item_string.as_str()) {
                    // Replace with value if item is a constant
                    Bucket::from_constant(*CONSTANT_IDENTIFIERS.get(item_string.as_str()).unwrap())
                } else if NUMERIC_REGEX.is_match(&item_string) {
                    Bucket::from(item_string.parse::<f64>().unwrap())
                } else {
                    Bucket::from(item_string)
                }
            }
        };

        // push the new item to the stack
        self.stack.push(item_pushable);

        Ok(EngineSignal::StackUpdated)
    }

    /// Get operands from stack as float
    pub fn get_operands_as_f(&mut self, number: i32) -> Result<Vec<f64>, String> {
        // Make sure there are actually enough items on the stack
        if self.stack.len() as i32 >= number {
            // Create vector to store operands
            let mut operands = Vec::new();
            // check that all items are of expected type
            let requested_operands = &self.stack[self.stack.len() - number as usize..];
            for item in requested_operands {
                match item.bucket_type {
                    BucketTypes::String | BucketTypes::Undefined => {
                        return Err(String::from(
                            "The operation cannot be performed on these operands",
                        ));
                    }
                    BucketTypes::Float | BucketTypes::Constant(_) => (),
                }
            }

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self.stack.pop().unwrap();

                // this is safe as we tested above for invalid variants
                operands.push(operand.value.unwrap().parse::<f64>().unwrap());
            }
            // Make the new vector's order match the stack
            operands.reverse();
            Ok(operands)
        } else {
            Err(String::from("Not enough items on stack for operation"))
        }
    }

    /// Get operands as a decimal object
    pub fn get_operands_as_dec(&mut self, number: i32) -> Result<Vec<Decimal>, String> {
        // Make sure there are actually enough items on the stack
        if self.stack.len() as i32 >= number {
            // Create vector to store operands
            let mut operands = Vec::new();
            // check that all items are of expected type
            let requested_operands = &self.stack[self.stack.len() - number as usize..];
            for item in requested_operands {
                match item.bucket_type {
                    BucketTypes::String | BucketTypes::Undefined => {
                        return Err(String::from(
                            "The operation cannot be performed on these operands",
                        ));
                    }
                    BucketTypes::Float | BucketTypes::Constant(_) => (),
                }
            }

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self.stack.pop().unwrap();
                operands.push(match operand.bucket_type {
                    BucketTypes::Constant(ConstantTypes::Pi) => Decimal::PI,
                    BucketTypes::Constant(ConstantTypes::E) => Decimal::E,
                    BucketTypes::Constant(ConstantTypes::HalfPi) => Decimal::HALF_PI,
                    BucketTypes::Constant(ConstantTypes::QuarterPi) => Decimal::QUARTER_PI,
                    BucketTypes::Constant(ConstantTypes::TwoPi) => Decimal::TWO_PI,
                    BucketTypes::Float
                    | BucketTypes::Constant(ConstantTypes::C)
                    | BucketTypes::Constant(ConstantTypes::G)
                    | BucketTypes::Constant(ConstantTypes::ThirdPi)
                    | BucketTypes::Constant(ConstantTypes::SixthPi)
                    | BucketTypes::Constant(ConstantTypes::EighthPi)
                    | BucketTypes::Constant(ConstantTypes::Phi) => {
                        match Decimal::from_str_exact(&operand.value.unwrap()) {
                            Ok(value) => value,
                            Err(e) => return Err(e.to_string()),
                        }
                    }
                    BucketTypes::String | BucketTypes::Undefined => {
                        return Err(String::from("you should never get this error"))
                    }
                });
            }
            // Make the new vector's order match the stack
            operands.reverse();
            Ok(operands)
        } else {
            Err(String::from("Not enough items on stack for operation"))
        }
    }

    /// Get operands as a string
    pub fn get_operands_as_string(&mut self, number: i32) -> Result<Vec<String>, String> {
        // Make sure there are actually enough items on the stack
        if self.stack.len() as i32 >= number {
            // Create vector to store operands
            let mut operands = Vec::new();
            // we can skip the type check since everything is already a string

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self.stack.pop().unwrap();

                operands.push(operand.to_string());
            }
            // Make the new vector's order match the stack
            operands.reverse();
            Ok(operands)
        } else {
            Err(String::from("Not enough items on stack for operation"))
        }
    }

    /// Get the raw Buckets from the stack
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

    /// Update the previous answer variable
    /// TODO: document that this function needs to be called a lot
    pub fn update_previous_answer(&mut self) -> Result<EngineSignal, String> {
        if !self.stack.is_empty() {
            self.previous_answer = self.stack.last().unwrap().clone();
            Ok(EngineSignal::NOP)
        } else {
            Err(String::from("stack is empty"))
        }
    }

    /// Add
    pub fn add(&mut self) -> Result<EngineSignal, String> {
        let operands = self.get_operands_as_dec(2)?;

        // Put result on stack
        let result = operands[0] + operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Subtract
    pub fn subtract(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_dec(2)?;

        // Put result on stack
        let result = operands[0] - operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Multiply
    pub fn multiply(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_dec(2)?;

        // manual handling for 2PI precision
        let check_pi = HashSet::from([Decimal::PI, dec!(2.0)]);
        let operands_set: HashSet<Decimal> = operands.clone().into_iter().collect();
        let non_matching_operands = check_pi
            .symmetric_difference(&operands_set)
            .collect::<Vec<_>>();

        let result = if non_matching_operands.is_empty() {
            // the only things on the mulitplication stack are 2 and pi, replace with the constant
            Bucket::from_constant(ConstantTypes::TwoPi)
        } else {
            // not 2*pi, perform normal mulitplication
            Bucket::from(operands[0] * operands[1])
        };
        // Put result on stack
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Divide
    pub fn divide(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_dec(2)?;

        if operands[1] == dec!(0.0) {
            return Err("cannot divide by 0".to_string());
        }

        // check for pi/x in order to replace with constants
        let result = if operands[0] == Decimal::PI {
            if operands[1] == dec!(2.0) {
                // pi/2
                Bucket::from_constant(ConstantTypes::HalfPi)
            } else if operands[1] == dec!(4.0) {
                // pi/4
                Bucket::from_constant(ConstantTypes::QuarterPi)
            } else if operands[1] == dec!(3.0) {
                // pi/3
                Bucket::from_constant(ConstantTypes::ThirdPi)
            } else if operands[1] == dec!(6.0) {
                // pi/6
                Bucket::from_constant(ConstantTypes::SixthPi)
            } else if operands[1] == dec!(8.0) {
                // pi/8
                Bucket::from_constant(ConstantTypes::EighthPi)
            } else {
                // denominator is not 2 or 4, eval normally
                Bucket::from(operands[0] / operands[1])
            }
        } else {
            // numerator is not pi, eval normally
            Bucket::from(operands[0] / operands[1])
        };

        // Put result on stack
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Power
    pub fn power(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_dec(2)?;

        let base = operands[0];
        let exponent = operands[1];

        // TODO: consider adding the option to use both rust_decimal and rug
        // detect if exponent is decimal, if so, don't use decimal library as that estimates
        let result = if exponent.fract() == dec!(0.0) {
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
        Ok(EngineSignal::StackUpdated)
    }

    /// Square root
    pub fn sqrt(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_dec(1)?;

        // Put result on stack
        let result = match operands[0].sqrt() {
            Some(value) => value,
            None => return Err("Error calculating sqrt".to_string()),
        };
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Modulo
    pub fn modulo(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = operands[0] % operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Sine
    pub fn sin(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let result = match operands[0].sin() {
            Some(value) => value,
            None => return Err("could not sin operand".to_string()),
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Cosine
    pub fn cos(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let result = match operands[0].cos() {
            Some(value) => value,
            None => return Err("could not cos operand".to_string()),
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Tangent
    pub fn tan(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_raw(1)?;
        // Put result on stack
        let result = match operands[0].tan() {
            Some(value) => value,
            None => return Err("could not tan operand".to_string()),
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Secant
    pub fn sec(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let result = match operands[0].sec() {
            Some(value) => value,
            None => return Err("could not sec operand".to_string()),
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Cosecant
    pub fn csc(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let result = match operands[0].csc() {
            Some(value) => value,
            None => return Err("could not csc operand".to_string()),
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Cotangent
    pub fn cot(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let result = match operands[0].cot() {
            Some(value) => value,
            None => return Err("could not sine operand".to_string()),
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Asin
    pub fn asin(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].asin().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Acos
    pub fn acos(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].acos().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Atan
    pub fn atan(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].atan().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Change sign
    pub fn chs(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let result = operands[0] * -1.0;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Logarithm
    pub fn log(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_dec(1)?;

        // Put result on stack
        let result = match operands[0].checked_log10() {
            Some(value) => value,
            None => return Err("cannot take log10 of 0 or negative numbers".to_string()),
        };
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Logarithm with custom base using the change of base formula
    pub fn blog(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_dec(2)?;

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

        let result = match top_log.checked_div(bottom_log) {
            Some(value) => value,
            None => return Err("cannot divide by zero".to_string()),
        };

        // Put result on stack
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Natural logarihm
    pub fn ln(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_dec(1)?;

        // Put result on stack
        let result = match operands[0].checked_ln() {
            Some(value) => value,
            None => return Err("cannot take log10 of 0 or negative numbers".to_string()),
        };
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Absolute value
    pub fn abs(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].abs().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Equal to
    pub fn eq(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        // TODO: maybe make this work with strings
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] == operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Greater than
    pub fn gt(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] > operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Less than
    pub fn lt(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] < operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Greater than or equal to
    pub fn geq(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] >= operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Less than or equal to
    pub fn leq(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] <= operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Round to nearest int
    pub fn round(&mut self) -> Result<EngineSignal, String> {
        // Get operand
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].round().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Calculate 1/x
    pub fn invert(&mut self) -> Result<EngineSignal, String> {
        // Get operand
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack((1_f64 / operands[0]).into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Drop last item from stack
    pub fn drop(&mut self) -> Result<EngineSignal, String> {
        // Remove last item from stack
        self.stack.pop();
        Ok(EngineSignal::StackUpdated)
    }

    /// Swap last two items on stack
    pub fn swap(&mut self) -> Result<EngineSignal, String> {
        // Get last two values from stack
        let operands = self.get_operands_raw(2)?;

        // Insert in reverse order
        let _ = self.add_item_to_stack(operands[1].clone());
        let _ = self.add_item_to_stack(operands[0].clone());
        Ok(EngineSignal::StackUpdated)
    }

    /// Duplicate the last item of the stack
    pub fn dup(&mut self) -> Result<EngineSignal, String> {
        // Get the last value from the stack
        let operands = self.get_operands_raw(1)?;

        // Insert twice
        let _ = self.add_item_to_stack(operands[0].clone());
        let _ = self.add_item_to_stack(operands[0].clone());
        Ok(EngineSignal::StackUpdated)
    }

    /// Roll down
    pub fn roll_down(&mut self) -> Result<EngineSignal, String> {
        if !self.stack.is_empty() {
            // Rotate stack right
            self.stack.rotate_right(1);
            Ok(EngineSignal::StackUpdated)
        } else {
            Err(String::from("Cannot roll empty stack"))
        }
    }

    /// Roll up
    pub fn roll_up(&mut self) -> Result<EngineSignal, String> {
        if !self.stack.is_empty() {
            // Rotate stack left
            self.stack.rotate_left(1);
            Ok(EngineSignal::StackUpdated)
        } else {
            Err(String::from("Cannot roll empty stack"))
        }
    }

    /// Store value in variable
    pub fn store(&mut self) -> Result<EngineSignal, String> {
        // Get 2 operands from stack
        let operands = self.get_operands_raw(2)?;

        // Only store if matches the identifier pattern
        let varname = operands[1].to_string();
        if ID_REGEX.is_match(&varname) {
            // Add variable to hashmap
            self.variables.insert(varname, operands[0].clone());
        } else {
            // Error if attempted to store in name which is not a valid ID
            return Err(format!("Cannot store in non-variable object `{}`", varname));
        }
        Ok(EngineSignal::StackUpdated)
    }

    /// Delete variable
    pub fn purge(&mut self) -> Result<EngineSignal, String> {
        // Get operand from stack
        let operands = self.get_operands_raw(1)?;

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
        Ok(EngineSignal::StackUpdated)
    }

    /// Store value in variable, with inverted argument order
    pub fn invstore(&mut self) -> Result<EngineSignal, String> {
        match self.swap() {
            Ok(_) => {}
            Err(error) => return Err(error),
        }
        self.store()
    }

    /// Clear stack
    pub fn clear(&mut self) -> Result<EngineSignal, String> {
        self.stack = Vec::new();
        Ok(EngineSignal::StackUpdated)
    }

    /// Update stack and variables from the undo history
    fn update_engine_from_history(&mut self) {
        self.stack =
            self.undo_history[self.undo_history.len() - self.undo_state_pointer as usize].clone();
        self.variables = self.undo_variable_history
            [self.undo_variable_history.len() - self.undo_state_pointer as usize]
            .clone();
    }

    /// Undo last operation
    pub fn undo(&mut self) -> Result<EngineSignal, String> {
        if self.undo_state_pointer < self.undo_history.len() as u8 {
            if self.undo_state_pointer == 0 {
                // add current stack and variables to hsitory and increment pointer by 1
                self.undo_history.push_back(self.stack.clone());
                self.undo_variable_history.push_back(self.variables.clone());
                self.undo_state_pointer += 1;
            }
            self.undo_state_pointer += 1;
            self.update_engine_from_history();
            Ok(EngineSignal::StackUpdated)
        } else {
            Err(String::from("Cannot undo further"))
        }
    }

    /// Redo the last undo
    pub fn redo(&mut self) -> Result<EngineSignal, String> {
        if self.undo_state_pointer > 1 {
            self.undo_state_pointer -= 1;
            self.update_engine_from_history();
            Ok(EngineSignal::StackUpdated)
        } else {
            Err(String::from("Cannot redo further"))
        }
    }

    // send quit code
    pub fn quit(&mut self) -> Result<EngineSignal, String> {
        Ok(EngineSignal::Quit)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
