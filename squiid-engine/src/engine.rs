use std::collections::{HashMap, HashSet, VecDeque};

use rust_decimal::{prelude::ToPrimitive, Decimal, MathematicalOps};
use rust_decimal_macros::dec;

use crate::{
    bucket::{Bucket, BucketTypes, ConstantTypes, CONSTANT_IDENTIFIERS},
    errors::{
        BaseLogError, CosError, CotError, CscError, DecimalReprError, DivisionError,
        EmptyStackError, EmptyStackRollError, FloatParseError, InvalidOperandTypeError,
        LogDomainError, MissingValueError, NotEnoughItemsError, OperandError, PopFailureError,
        PowerError, PurgeError, RedoLimitError, SecError, SinError, SqrtError,
        StoreVariableInvalidIDError, TanError, UndefinedVariableReferenceError, UndoLimitError,
        ZeroDivisionError,
    },
    utils::ID_REGEX,
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
    /// Index will be calculated by `history.len() - pointer - 1`
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

    /// Add an item to the stack
    ///
    /// # Arguments
    ///
    /// * `item` - the item to add to the stack
    ///
    /// # Errors
    ///
    /// TODO:
    pub fn add_item_to_stack(
        &mut self,
        item: Bucket,
    ) -> Result<EngineSignal, UndefinedVariableReferenceError> {
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
                None => return Err(UndefinedVariableReferenceError(item_string)),
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
                match CONSTANT_IDENTIFIERS.get(item_string.as_str()) {
                    Some(&constant) => Bucket::from_constant(constant),
                    None => match item_string.parse::<f64>() {
                        Ok(val) => Bucket::from(val),
                        Err(_) => Bucket::from(item_string),
                    },
                }
            }
        };

        // push the new item to the stack
        self.stack.push(item_pushable);

        Ok(EngineSignal::StackUpdated)
    }

    /// Get operands from stack as float
    pub fn get_operands_as_f(&mut self, number: i32) -> Result<Vec<f64>, OperandError> {
        // Make sure there are actually enough items on the stack
        if self.stack.len() as i32 >= number {
            // Create vector to store operands
            let mut operands = Vec::new();
            // check that all items are of expected type
            let requested_operands = &self.stack[self.stack.len() - number as usize..];
            for item in requested_operands {
                match item.bucket_type {
                    BucketTypes::String | BucketTypes::Undefined => {
                        return Err(InvalidOperandTypeError.into());
                    }
                    BucketTypes::Float | BucketTypes::Constant(_) => (),
                }
            }

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self.stack.pop().ok_or_else(|| PopFailureError)?;

                // this is safe as we tested above for invalid variants
                let value = operand.value.ok_or_else(|| MissingValueError)?;
                operands.push(value.parse::<f64>().map_err(|e| FloatParseError(e))?);
            }
            // Make the new vector's order match the stack
            operands.reverse();
            Ok(operands)
        } else {
            Err(NotEnoughItemsError.into())
        }
    }

    /// Get operands as a decimal object
    pub fn get_operands_as_dec(&mut self, number: i32) -> Result<Vec<Decimal>, OperandError> {
        // Make sure there are actually enough items on the stack
        if self.stack.len() as i32 >= number {
            // Create vector to store operands
            let mut operands = Vec::new();
            // check that all items are of expected type
            let requested_operands = &self.stack[self.stack.len() - number as usize..];
            for item in requested_operands {
                match item.bucket_type {
                    BucketTypes::String | BucketTypes::Undefined => {
                        return Err(InvalidOperandTypeError.into());
                    }
                    BucketTypes::Float | BucketTypes::Constant(_) => (),
                }
            }

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self.stack.pop().ok_or_else(|| PopFailureError)?;
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
                        match Decimal::from_str_exact(
                            &operand
                                .value
                                .ok_or_else(|| MissingValueError)?,
                        ) {
                            Ok(value) => value,
                            Err(e) => return Err(DecimalReprError(e).into()),
                        }
                    }
                    BucketTypes::String | BucketTypes::Undefined => {
                        unreachable!("we've already checked that each operand on the stack is not an invalid type: operands as dec")
                    }
                });
            }
            // Make the new vector's order match the stack
            operands.reverse();
            Ok(operands)
        } else {
            Err(NotEnoughItemsError.into())
        }
    }

    /// Get operands as a string
    pub fn get_operands_as_string(&mut self, number: i32) -> Result<Vec<String>, OperandError> {
        // Make sure there are actually enough items on the stack
        if self.stack.len() as i32 >= number {
            // Create vector to store operands
            let mut operands = Vec::new();
            // we can skip the type check since everything is already a string

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self.stack.pop().ok_or_else(|| PopFailureError)?;

                operands.push(operand.to_string());
            }
            // Make the new vector's order match the stack
            operands.reverse();
            Ok(operands)
        } else {
            Err(NotEnoughItemsError.into())
        }
    }

    /// Get the raw Buckets from the stack
    pub fn get_operands_raw(&mut self, number: i32) -> Result<Vec<Bucket>, OperandError> {
        if self.stack.len() as i32 >= number {
            // Create vector to store operands
            let mut operands = Vec::new();

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self.stack.pop().ok_or_else(|| PopFailureError)?;

                operands.push(operand);
            }
            // Make the new vector's order match the stack
            operands.reverse();
            Ok(operands)
        } else {
            Err(NotEnoughItemsError.into())
        }
    }

    /// Update the previous answer variable
    /// TODO: document that this function needs to be called a lot
    pub fn update_previous_answer(&mut self) -> Result<EngineSignal, EmptyStackError> {
        match self.stack.last() {
            Some(last) => {
                self.previous_answer = last.clone();
                Ok(EngineSignal::NOP)
            }
            None => Err(EmptyStackError),
        }
    }

    /// Add
    pub fn add(&mut self) -> Result<EngineSignal, OperandError> {
        let operands = self.get_operands_as_dec(2)?;

        // Put result on stack
        let result = operands[0] + operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Subtract
    pub fn subtract(&mut self) -> Result<EngineSignal, OperandError> {
        // Get operands
        let operands = self.get_operands_as_dec(2)?;

        // Put result on stack
        let result = operands[0] - operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Multiply
    pub fn multiply(&mut self) -> Result<EngineSignal, OperandError> {
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
    pub fn divide(&mut self) -> Result<EngineSignal, DivisionError> {
        // Get operands
        let operands = self.get_operands_as_dec(2)?;

        if operands[1] == dec!(0.0) {
            return Err(ZeroDivisionError.into());
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
    pub fn power(&mut self) -> Result<EngineSignal, PowerError> {
        // Get operands
        let operands = self.get_operands_as_dec(2)?;

        let base = operands[0];
        let exponent = operands[1];

        // TODO: consider adding the option to use both rust_decimal and rug
        // detect if exponent is decimal, if so, don't use decimal library as that estimates
        let result = if exponent.fract() == dec!(0.0) {
            // is not a decimal
            match base.checked_powd(exponent) {
                Some(value) => value
                    .to_f64()
                    .ok_or_else(|| PowerError::FloatConversionError(value.to_string()))?,
                None => return Err(PowerError::PowerOverflowError),
            }
        } else {
            // is a decimal
            let exponent = exponent
                .to_f64()
                .ok_or_else(|| PowerError::FloatConversionError(exponent.to_string()))?;
            base.to_f64()
                .ok_or_else(|| PowerError::FloatConversionError(exponent.to_string()))?
                .powf(exponent)
        };

        // Put result on stack
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Square root
    pub fn sqrt(&mut self) -> Result<EngineSignal, SqrtError> {
        // Get operands
        let operands = self.get_operands_as_dec(1)?;

        // Put result on stack
        let Some(result) = operands[0].sqrt() else {
            return Err(SqrtError::SqrtError);
        };
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Modulo (euclidean)
    pub fn modulo(&mut self) -> Result<EngineSignal, DivisionError> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        if operands[1] == 0.0 {
            return Err(ZeroDivisionError.into());
        }

        // Put result on stack
        // rem_euclid() only yields positive results so we need to write it ourselves
        let r = operands[0] % operands[1];
        let result = if (r < 0.0 && operands[1] > 0.0) || (r > 0.0 && operands[1] < 0.0) {
            r + operands[1]
        } else {
            r
        };
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Sine
    pub fn sin(&mut self) -> Result<EngineSignal, SinError> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let Some(result) = operands[0].sin() else {
            return Err(SinError::SinError);
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Cosine
    pub fn cos(&mut self) -> Result<EngineSignal, CosError> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let Some(result) = operands[0].cos() else {
            return Err(CosError::CosError);
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Tangent
    pub fn tan(&mut self) -> Result<EngineSignal, TanError> {
        // Get operands
        let operands = self.get_operands_raw(1)?;
        // Put result on stack
        let Some(result) = operands[0].tan() else {
            return Err(TanError::TanError);
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Secant
    pub fn sec(&mut self) -> Result<EngineSignal, SecError> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let Some(result) = operands[0].sec() else {
            return Err(SecError::SecError);
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Cosecant
    pub fn csc(&mut self) -> Result<EngineSignal, CscError> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let Some(result) = operands[0].csc() else {
            return Err(CscError::CscError);
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Cotangent
    pub fn cot(&mut self) -> Result<EngineSignal, CotError> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let Some(result) = operands[0].cot() else {
            return Err(CotError::CotError);
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Asin
    pub fn asin(&mut self) -> Result<EngineSignal, OperandError> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].asin().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Acos
    pub fn acos(&mut self) -> Result<EngineSignal, OperandError> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].acos().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Atan
    pub fn atan(&mut self) -> Result<EngineSignal, OperandError> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].atan().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Change sign
    pub fn chs(&mut self) -> Result<EngineSignal, OperandError> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let result = operands[0] * -1.0;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Logarithm
    pub fn log(&mut self) -> Result<EngineSignal, LogDomainError> {
        // Get operands
        let operands = self.get_operands_as_dec(1)?;

        // Put result on stack
        let Some(result) = operands[0].checked_log10() else {
            return Err(LogDomainError::LogDomainError.into());
        };
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Logarithm with custom base using the change of base formula
    pub fn blog(&mut self) -> Result<EngineSignal, BaseLogError> {
        // Get operands
        let operands = self.get_operands_as_dec(2)?;

        // change of base formula is defined as follows:
        // log_b(a) = (log_d(a))/(log_d(b))

        let Some(top_log) = operands[0].checked_log10() else {
            return Err(BaseLogError::LogDomainError);
        };
        let Some(bottom_log) = operands[1].checked_log10() else {
            return Err(BaseLogError::LogBaseDomainError);
        };

        let Some(result) = top_log.checked_div(bottom_log) else {
            return Err(ZeroDivisionError.into());
        };

        // Put result on stack
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Natural logarihm
    pub fn ln(&mut self) -> Result<EngineSignal, LogDomainError> {
        // Get operands
        let operands = self.get_operands_as_dec(1)?;

        // Put result on stack
        let Some(result) = operands[0].checked_ln() else {
            return Err(LogDomainError::LogDomainError.into());
        };
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Absolute value
    pub fn abs(&mut self) -> Result<EngineSignal, OperandError> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].abs().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Equal to
    pub fn eq(&mut self) -> Result<EngineSignal, OperandError> {
        // Get operands
        // TODO: maybe make this work with strings
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] == operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Greater than
    pub fn gt(&mut self) -> Result<EngineSignal, OperandError> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] > operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Less than
    pub fn lt(&mut self) -> Result<EngineSignal, OperandError> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] < operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Greater than or equal to
    pub fn geq(&mut self) -> Result<EngineSignal, OperandError> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] >= operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Less than or equal to
    pub fn leq(&mut self) -> Result<EngineSignal, OperandError> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] <= operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Round to nearest int
    pub fn round(&mut self) -> Result<EngineSignal, OperandError> {
        // Get operand
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].round().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Calculate 1/x
    pub fn invert(&mut self) -> Result<EngineSignal, OperandError> {
        // Get operand
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack((1_f64 / operands[0]).into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Drop last item from stack
    pub fn drop(&mut self) -> Result<EngineSignal, OperandError> {
        // Remove last item from stack
        self.stack.pop();
        Ok(EngineSignal::StackUpdated)
    }

    /// Swap last two items on stack
    pub fn swap(&mut self) -> Result<EngineSignal, OperandError> {
        // Get last two values from stack
        let operands = self.get_operands_raw(2)?;

        // Insert in reverse order
        let _ = self.add_item_to_stack(operands[1].clone());
        let _ = self.add_item_to_stack(operands[0].clone());
        Ok(EngineSignal::StackUpdated)
    }

    /// Duplicate the last item of the stack
    pub fn dup(&mut self) -> Result<EngineSignal, OperandError> {
        // Get the last value from the stack
        let operands = self.get_operands_raw(1)?;

        // Insert twice
        let _ = self.add_item_to_stack(operands[0].clone());
        let _ = self.add_item_to_stack(operands[0].clone());
        Ok(EngineSignal::StackUpdated)
    }

    /// Roll down
    pub fn roll_down(&mut self) -> Result<EngineSignal, EmptyStackRollError> {
        if self.stack.is_empty() {
            Err(EmptyStackRollError)
        } else {
            // Rotate stack right
            self.stack.rotate_right(1);
            Ok(EngineSignal::StackUpdated)
        }
    }

    /// Roll up
    pub fn roll_up(&mut self) -> Result<EngineSignal, EmptyStackRollError> {
        if self.stack.is_empty() {
            Err(EmptyStackRollError)
        } else {
            // Rotate stack left
            self.stack.rotate_left(1);
            Ok(EngineSignal::StackUpdated)
        }
    }

    /// Store value in variable
    pub fn store(&mut self) -> Result<EngineSignal, StoreVariableInvalidIDError> {
        // Get 2 operands from stack
        let operands = self.get_operands_raw(2)?;

        // Only store if matches the identifier pattern
        let varname = operands[1].to_string();
        if ID_REGEX.is_match(&varname) {
            // Add variable to hashmap
            self.variables.insert(varname, operands[0].clone());
        } else {
            // Error if attempted to store in name which is not a valid ID
            return Err(StoreVariableInvalidIDError::StoreVariableInvalidIDError(varname).into());
        }
        Ok(EngineSignal::StackUpdated)
    }

    /// Delete variable
    pub fn purge(&mut self) -> Result<EngineSignal, PurgeError> {
        // Get operand from stack
        let operands = self.get_operands_raw(1)?;

        let varname = operands[0].to_string();
        if ID_REGEX.is_match(&varname) {
            if self.variables.contains_key(&varname) {
                // Remove variable from hashmap
                self.variables.remove(&varname);
            } else {
                return Err(PurgeError::NonExistantVariableError(varname));
            }
        } else {
            // Error if attempted to purge name which is not a valid ID
            return Err(PurgeError::DeleteVariableInvalidIDError(varname));
        }
        Ok(EngineSignal::StackUpdated)
    }

    /// Store value in variable, with inverted argument order
    pub fn invstore(&mut self) -> Result<EngineSignal, StoreVariableInvalidIDError> {
        match self.swap() {
            Ok(_) => {}
            Err(error) => return Err(error.into()),
        }
        self.store()
    }

    /// Clear stack
    pub fn clear(&mut self) -> Result<EngineSignal, OperandError> {
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
    pub fn undo(&mut self) -> Result<EngineSignal, UndoLimitError> {
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
            Err(UndoLimitError)
        }
    }

    /// Redo the last undo
    pub fn redo(&mut self) -> Result<EngineSignal, RedoLimitError> {
        if self.undo_state_pointer > 1 {
            self.undo_state_pointer -= 1;
            self.update_engine_from_history();
            Ok(EngineSignal::StackUpdated)
        } else {
            Err(RedoLimitError)
        }
    }

    // send quit code
    pub fn quit(&mut self) -> Result<EngineSignal, OperandError> {
        Ok(EngineSignal::Quit)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
