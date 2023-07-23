use std::collections::{HashMap, HashSet, VecDeque};

use rust_decimal::{prelude::ToPrimitive, Decimal, MathematicalOps};
use rust_decimal_macros::dec;

use crate::{
    bucket::{Bucket, BucketTypes, ConstantTypes},
    utils::{ID_REGEX, NUMERIC_REGEX},
    MessageAction,
};

/// Evaluation engine struct
pub struct Engine {
    pub stack: Vec<Bucket>,
    pub variables: HashMap<String, Bucket>,
    pub history: VecDeque<Vec<Bucket>>,
    pub variable_history: VecDeque<HashMap<String, Bucket>>,
    pub previous_answer: Bucket,
}

/// Evaluation engine implementation
impl Engine {
    /// Helper to construct a new engine object
    pub fn new() -> Engine {
        Engine {
            /// The stack of bucket items
            stack: Vec::new(),
            /// Hashmap of set variables
            variables: HashMap::new(),
            /// History vecdeque for undo support
            history: VecDeque::new(),
            /// Variables vecdeque for undo support
            variable_history: VecDeque::new(),
            /// Previous answer
            previous_answer: Bucket::from(0),
        }
    }

    /// Add item to stack
    pub fn add_item_to_stack(
        &mut self,
        item: Bucket,
    ) -> Result<MessageAction, String> {
        // Convert item to string
        let mut item_string = item.to_string();

        // substitute previous answer
        if item_string == "@" {
            item_string = self.previous_answer.to_string();
        }

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

        // Replace with value if item is a constant
        let exposed_constants = HashMap::from([
            ("#pi", ConstantTypes::PI),
            ("#e", ConstantTypes::E),
            ("#tau", ConstantTypes::TAU),
            ("#c", ConstantTypes::C),
            ("#G", ConstantTypes::G),
        ]);

        // create a StackableFloat if item_string is numeric, else StackableString
        let item_pushable: Bucket = match item.bucket_type {
            BucketTypes::Undefined => Bucket::new_undefined(),
            BucketTypes::Constant(constant_type) => {
                // bucket already has a constant type, use that
                Bucket::from_constant(constant_type)
            }
            _ => {
                // test all other options
                if exposed_constants.contains_key(item_string.as_str()) {
                    Bucket::from_constant(
                        exposed_constants.get(item_string.as_str()).unwrap().clone(),
                    )
                } else if NUMERIC_REGEX.is_match(&item_string) {
                    Bucket::from(item_string.parse::<f64>().unwrap())
                } else {
                    Bucket::from(item_string)
                }
            }
        };

        // push the new item to the stack
        self.stack.push(item_pushable);

        Ok(MessageAction::SendStack)
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
                    _ => (),
                }
            }

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self.stack.pop().unwrap();

                operands.push(match operand.bucket_type {
                    BucketTypes::Float | BucketTypes::Constant(_) => {
                        operand.value.unwrap().parse::<f64>().unwrap()
                    }
                    _ => return Err(String::from("you should never get this error")),
                });
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
                    _ => (),
                }
            }

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self.stack.pop().unwrap();
                operands.push(match operand.bucket_type {
                    BucketTypes::Constant(ConstantTypes::PI) => Decimal::PI,
                    BucketTypes::Constant(ConstantTypes::E) => Decimal::E,
                    BucketTypes::Constant(ConstantTypes::HalfPI) => Decimal::HALF_PI,
                    BucketTypes::Constant(ConstantTypes::QuarterPI) => Decimal::QUARTER_PI,
                    BucketTypes::Constant(ConstantTypes::TwoPI) => Decimal::TWO_PI,
                    BucketTypes::Float
                    | BucketTypes::Constant(ConstantTypes::TAU)
                    | BucketTypes::Constant(ConstantTypes::C)
                    | BucketTypes::Constant(ConstantTypes::G)
                    | BucketTypes::Constant(ConstantTypes::ThirdPI)
                    | BucketTypes::Constant(ConstantTypes::SixthPI)
                    | BucketTypes::Constant(ConstantTypes::EighthPI) => {
                        Decimal::from_str_exact(&operand.value.unwrap()).unwrap()
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

    /// Add
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

    /// Subtract
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

    /// Multiply
    pub fn multiply(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_dec(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // manual handling for 2PI precision
        let check_pi = HashSet::from([Decimal::PI, dec!(2.0)]);
        let operands_set: HashSet<Decimal> = operands.clone().into_iter().collect();
        let non_matching_operands = check_pi
            .symmetric_difference(&operands_set)
            .into_iter()
            .collect::<Vec<_>>();

        let result = if non_matching_operands.is_empty() {
            // the only things on the mulitplication stack are 2 and pi, replace with the constant
            Bucket::from_constant(ConstantTypes::TwoPI)
        } else {
            // not 2*pi, perform normal mulitplication
            Bucket::from(operands[0] * operands[1])
        };

        // Put result on stack
        let _ = self.add_item_to_stack(result);
        Ok(MessageAction::SendStack)
    }

    /// Divide
    pub fn divide(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_as_dec(2) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        if operands[1] == dec!(0.0) {
            return Err("cannot divide by 0".to_string());
        }

        // check for pi/x in order to replace with constants
        let result = if operands[0] == Decimal::PI {
            if operands[1] == dec!(2.0) {
                // pi/2
                Bucket::from_constant(ConstantTypes::HalfPI)
            } else if operands[1] == dec!(4.0) {
                // pi/4
                Bucket::from_constant(ConstantTypes::QuarterPI)
            } else if operands[1] == dec!(3.0) {
                // pi/3
                Bucket::from_constant(ConstantTypes::ThirdPI)
            } else if operands[1] == dec!(6.0) {
                // pi/6
                Bucket::from_constant(ConstantTypes::SixthPI)
            } else if operands[1] == dec!(8.0) {
                // pi/8
                Bucket::from_constant(ConstantTypes::EighthPI)
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
        Ok(MessageAction::SendStack)
    }

    /// Power
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
        Ok(MessageAction::SendStack)
    }

    /// Square root
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

    /// Modulo
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

    /// Sine
    pub fn sin(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_raw(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = match operands[0].sin() {
            Some(value) => value,
            None => return Err("could not sin operand".to_string()),
        };
        let _ = self.add_item_to_stack(result);
        Ok(MessageAction::SendStack)
    }

    /// Cosine
    pub fn cos(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_raw(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = match operands[0].cos() {
            Some(value) => value,
            None => return Err("could not cos operand".to_string()),
        };
        let _ = self.add_item_to_stack(result);
        Ok(MessageAction::SendStack)
    }

    /// Tangent
    pub fn tan(&mut self) -> Result<MessageAction, String> {
        // Get operands
        // TODO: undefined handling
        let operands = match self.get_operands_raw(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };
        // Put result on stack
        let result = match operands[0].tan() {
            Some(value) => value,
            None => return Err("could not tan operand".to_string()),
        };
        let _ = self.add_item_to_stack(result);
        Ok(MessageAction::SendStack)
    }

    /// Secant
    pub fn sec(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_raw(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = match operands[0].sec() {
            Some(value) => value,
            None => return Err("could not sec operand".to_string()),
        };
        let _ = self.add_item_to_stack(result);
        Ok(MessageAction::SendStack)
    }

    /// Cosecant
    pub fn csc(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_raw(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = match operands[0].csc() {
            Some(value) => value,
            None => return Err("could not csc operand".to_string()),
        };
        let _ = self.add_item_to_stack(result);
        Ok(MessageAction::SendStack)
    }

    /// Cotangent
    pub fn cot(&mut self) -> Result<MessageAction, String> {
        // Get operands
        let operands = match self.get_operands_raw(1) {
            Ok(content) => content,
            Err(error) => return Err(error),
        };

        // Put result on stack
        let result = match operands[0].cot() {
            Some(value) => value,
            None => return Err("could not sine operand".to_string()),
        };
        let _ = self.add_item_to_stack(result);
        Ok(MessageAction::SendStack)
    }

    /// Asin
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

    /// Acos
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

    /// Atan
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

    /// Change sign
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

    /// Logarithm
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

    /// Logarithm with custom base using the change of base formula
    pub fn blog(&mut self) -> Result<MessageAction, String> {
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

    /// Natural logarihm
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

    /// Absolute value
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

    /// Equal to
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

    /// Greater than
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

    /// Less than
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

    /// Greater than or equal to
    pub fn egt(&mut self) -> Result<MessageAction, String> {
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

    /// Less than or equal to
    pub fn elt(&mut self) -> Result<MessageAction, String> {
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

    /// Round to nearest int
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

    /// Calculate 1/x
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

    /// Drop last item from stack
    pub fn drop(&mut self) -> Result<MessageAction, String> {
        // Remove last item from stack
        self.stack.pop();
        Ok(MessageAction::SendStack)
    }

    /// Swap last two items on stack
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

    /// Duplicate the last item of the stack
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

    /// Roll down
    pub fn roll_down(&mut self) -> Result<MessageAction, String> {
        if self.stack.len() > 0 {
            // Rotate stack right
            self.stack.rotate_right(1);
            Ok(MessageAction::SendStack)
        } else {
            Err(String::from("Cannot roll empty stack"))
        }
    }

    /// Roll up
    pub fn roll_up(&mut self) -> Result<MessageAction, String> {
        if self.stack.len() > 0 {
            // Rotate stack left
            self.stack.rotate_left(1);
            Ok(MessageAction::SendStack)
        } else {
            Err(String::from("Cannot roll empty stack"))
        }
    }

    /// Store value in variable
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

    /// Delete variable
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

    /// Store value in variable, with inverted argument order
    pub fn invstore(&mut self) -> Result<MessageAction, String> {
        match self.swap() {
            Ok(_) => {}
            Err(error) => return Err(error),
        }
        self.store()
    }

    /// Clear stack
    pub fn clear(&mut self) -> Result<MessageAction, String> {
        self.stack = Vec::new();
        Ok(MessageAction::SendStack)
    }

    /// Undo last operation
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

    /// Send a list of commands to the client
    pub fn list_commands(&mut self) -> Result<MessageAction, String> {
        Ok(MessageAction::SendCommands)
    }

    // send quit code
    pub fn quit(&mut self) -> Result<MessageAction, String> {
        Ok(MessageAction::Quit)
    }
}
