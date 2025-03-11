use std::collections::{HashMap, HashSet, VecDeque};

use rust_decimal::{prelude::ToPrimitive, Decimal, MathematicalOps};
use rust_decimal_macros::dec;

use crate::{
    bucket::{Bucket, BucketTypes, ConstantTypes, CONSTANT_IDENTIFIERS},
    utils::ID_REGEX,
    EngineSignal,
};

/// The core evaluation engine responsible for processing Reverse Polish Notation (RPN) operations.
///
/// The [`Engine`] maintains a stack, variable storage, and undo history to facilitate command execution
/// and state management.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Engine {
    /// Initializes an empty stack, variable storage, and undo history, with the previous answer set to zero.
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

    /// Adds an item to the stack, resolving variables and constants if necessary.
    ///
    /// # Arguments
    ///
    /// * `item` - A [`Bucket`] containing the value to be added to the stack.
    ///
    /// # Behavior
    ///
    /// - If `item` is `"@"`, it is replaced with the value of `self.previous_answer`.
    /// - If `item` is a variable (its string representation starts with `$`), it is resolved using `self.variables`.
    ///   - If the variable is undefined, an error is returned.
    /// - If `item` is a recognized constant, it is converted accordingly.
    /// - If `item` is a numeric string, it is parsed into a `Float` bucket.
    /// - Otherwise, `item` is stored as a `String` bucket.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the item is successfully added to the stack.
    /// - `Err(String)` if an undefined variable is referenced.
    ///
    /// # Errors
    ///
    /// Returns an error if the function encounters a reference to an undefined variable.
    ///
    /// # Side Effects
    ///
    /// - Modifies `self.stack` by pushing the resolved `Bucket`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.variables.insert("x".to_string(), Bucket::from(42.0));
    /// assert!(engine.add_item_to_stack(Bucket::from("$x")).is_ok()); // Pushes 42.0 to the stack.
    /// assert!(engine.add_item_to_stack(Bucket::from("$y")).is_err()); // Error: Undefined variable.
    /// ```
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

    /// Retrieves a specified number of operands from the stack as `f64` values.
    ///
    /// # Arguments
    ///
    /// * `number` - The number of operands to retrieve from the stack.
    ///
    /// # Behavior
    ///
    /// - Ensures that the stack contains at least `number` elements.
    /// - Checks that all requested operands are valid types ([`BucketTypes::Float`] or [`BucketTypes::Constant`]).
    /// - Extracts the operands from the stack, converting them to `f64`.
    /// - Returns the operands in the correct order.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<f64>)` containing the extracted operands.
    /// - `Err(String)` if:
    ///   - The stack does not contain enough items.
    ///   - Any of the operands are of an invalid type ([`BucketTypes::String`] or [`BucketTypes::Undefined`]).
    ///   - An operand is missing a value.
    ///   - An operand cannot be parsed as `f64`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The stack has fewer items than `number`.
    /// - An operand is of an incompatible type.
    /// - Parsing an operand as `f64` fails.
    ///
    /// # Side Effects
    ///
    /// - Modifies `self.stack` by removing the retrieved operands.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(3.5));
    /// engine.stack.push(Bucket::from(2.0));
    ///
    /// let result = engine.get_operands_as_f(2);
    /// assert_eq!(result, Ok(vec![3.5, 2.0])); // Successfully retrieved operands.
    /// ```
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
                let operand = self
                    .stack
                    .pop()
                    .ok_or_else(|| String::from("Failed to pop operand"))?;

                // this is safe as we tested above for invalid variants
                let value = operand
                    .value
                    .ok_or_else(|| String::from("Operand value is missing"))?;
                operands.push(
                    value
                        .parse::<f64>()
                        .map_err(|e| format!("Failed to parse operand as f64: {}", e))?,
                );
            }
            // Make the new vector's order match the stack
            operands.reverse();
            Ok(operands)
        } else {
            Err(String::from("Not enough items on stack for operation"))
        }
    }

    /// Retrieves a specified number of operands from the stack as [`Decimal`] values.
    ///
    /// # Arguments
    ///
    /// * `number` - The number of operands to retrieve from the stack.
    ///
    /// # Behavior
    ///
    /// - Ensures that the stack contains at least `number` elements.
    /// - Checks that all requested operands are valid types ([`BucketTypes::Float`] or [`BucketTypes::Constant`]).
    /// - Converts recognized mathematical constants (`π`, `e`, etc.) into their corresponding [`Decimal`] values.
    /// - Parses other valid numeric values into [`Decimal`].
    /// - Returns the operands in the correct order.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Decimal>)` containing the extracted operands.
    /// - `Err(String)` if:
    ///   - The stack does not contain enough items.
    ///   - Any of the operands are of an invalid type ([`BucketTypes::String`] or [`BucketTypes::Undefined`]).
    ///   - An operand is missing a value.
    ///   - Parsing an operand as [`Decimal`] fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The stack has fewer items than `number`.
    /// - An operand is of an incompatible type.
    /// - Parsing an operand as [`Decimal`] fails.
    ///
    /// # Side Effects
    ///
    /// - Modifies `self.stack` by removing the retrieved operands.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(3.1415926535));
    /// engine.stack.push(Bucket::from(2.0));
    ///
    /// let result = engine.get_operands_as_dec(2);
    /// assert!(result.is_ok()); // Successfully retrieved operands as Decimals.
    /// ```
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
                let operand = self
                    .stack
                    .pop()
                    .ok_or_else(|| String::from("Failed to pop operand"))?;
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
                                .ok_or_else(|| String::from("Operand value is missing"))?,
                        ) {
                            Ok(value) => value,
                            Err(e) => return Err(e.to_string()),
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
            Err(String::from("Not enough items on stack for operation"))
        }
    }

    /// Retrieves a specified number of operands from the stack as [`String`] values.
    ///
    /// # Arguments
    ///
    /// * `number` - The number of operands to retrieve from the stack.
    ///
    /// # Behavior
    ///
    /// - Ensures that the stack contains at least `number` elements.
    /// - Converts each operand to a [`String`].
    /// - Maintains the correct order of retrieved operands.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<String>)` containing the extracted operands as strings.
    /// - `Err(String)` if there are not enough items on the stack.
    ///
    /// # Errors
    ///
    /// Returns an error if the stack has fewer items than `number`.
    ///
    /// # Side Effects
    ///
    /// - Modifies `self.stack` by removing the retrieved operands.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from("hello"));
    /// engine.stack.push(Bucket::from("world"));
    ///
    /// let result = engine.get_operands_as_string(2);
    /// assert_eq!(result.unwrap(), vec!["hello", "world"]);
    /// ```
    pub fn get_operands_as_string(&mut self, number: i32) -> Result<Vec<String>, String> {
        // Make sure there are actually enough items on the stack
        if self.stack.len() as i32 >= number {
            // Create vector to store operands
            let mut operands = Vec::new();
            // we can skip the type check since everything is already a string

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self
                    .stack
                    .pop()
                    .ok_or_else(|| String::from("Failed to pop operand"))?;

                operands.push(operand.to_string());
            }
            // Make the new vector's order match the stack
            operands.reverse();
            Ok(operands)
        } else {
            Err(String::from("Not enough items on stack for operation"))
        }
    }

    /// Retrieves a specified number of operands from the stack as raw [`Bucket`] values.
    ///
    /// # Arguments
    ///
    /// * `number` - The number of operands to retrieve from the stack.
    ///
    /// # Behavior
    ///
    /// - Ensures that the stack contains at least `number` elements.
    /// - Extracts the top `number` elements from the stack without modifying their types.
    /// - Maintains the original order of retrieved operands.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Bucket>)` containing the extracted operands in their raw form.
    /// - `Err(String)` if there are not enough items on the stack.
    ///
    /// # Errors
    ///
    /// Returns an error if the stack has fewer items than `number`.
    ///
    /// # Side Effects
    ///
    /// - Modifies `self.stack` by removing the retrieved operands.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(3.14));
    /// engine.stack.push(Bucket::from("test"));
    ///
    /// let result = engine.get_operands_raw(2);
    /// assert!(result.is_ok());
    /// assert_eq!(result.unwrap(), vec![Bucket::from(3.14), Bucket::from("test")]);
    /// ```
    pub fn get_operands_raw(&mut self, number: i32) -> Result<Vec<Bucket>, String> {
        if self.stack.len() as i32 >= number {
            // Create vector to store operands
            let mut operands = Vec::new();

            // Add requested number of operands from stack to vector and converts them to strings
            for _ in 0..number {
                let operand = self
                    .stack
                    .pop()
                    .ok_or_else(|| String::from("Failed to pop operand"))?;

                operands.push(operand);
            }
            // Make the new vector's order match the stack
            operands.reverse();
            Ok(operands)
        } else {
            Err(String::from("Not enough items on stack for operation"))
        }
    }

    /// Updates the `previous_answer` variable to the last item on the stack.
    ///
    /// # Behavior
    ///
    /// - If the stack is not empty, the last item is cloned and stored as `previous_answer`.
    /// - If the stack is empty, returns an error.
    ///
    /// # Algebraic Mode Consideration
    ///
    /// - If you're application has an algebraic mode, this function **must** be called **after** a full
    ///   algebraic expression has been executed. This ensures that `previous_answer` is updated at the
    ///   **end** of the statement, rather than in the middle of execution.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::NOP)` if `previous_answer` is successfully updated.
    /// - `Err(String)` if the stack is empty.
    ///
    /// # Errors
    ///
    /// Returns an error if the stack is empty, as there is no value to store as `previous_answer`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(42));
    ///
    /// assert!(engine.update_previous_answer().is_ok());
    /// assert_eq!(engine.previous_answer, Bucket::from(42));
    /// ```
    pub fn update_previous_answer(&mut self) -> Result<EngineSignal, String> {
        match self.stack.last() {
            Some(last) => {
                self.previous_answer = last.clone();
                Ok(EngineSignal::NOP)
            }
            None => Err(String::from("stack is empty")),
        }
    }

    /// Performs addition on the top two operands of the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top two values from the stack as [`Decimal`] numbers.
    /// - Computes the sum of these values.
    /// - Pushes the result back onto the stack.
    /// - Returns [`EngineSignal::StackUpdated`] to indicate a successful operation.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if there are not enough operands on the stack or if the operands are invalid.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains fewer than two operands.
    /// - The operands are of an invalid type that cannot be converted to [`Decimal`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(5));
    /// engine.stack.push(Bucket::from(10));
    ///
    /// assert!(engine.add().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(15));
    /// ```
    pub fn add(&mut self) -> Result<EngineSignal, String> {
        let operands = self.get_operands_as_dec(2)?;

        // Put result on stack
        let result = operands[0] + operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Performs subtraction on the top two operands of the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top two values from the stack as [`Decimal`] numbers.
    /// - Computes the difference of these values.
    /// - Pushes the result back onto the stack.
    /// - Returns [`EngineSignal::StackUpdated`] to indicate a successful operation.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if there are not enough operands on the stack or if the operands are invalid.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains fewer than two operands.
    /// - The operands are of an invalid type that cannot be converted to [`Decimal`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(5));
    /// engine.stack.push(Bucket::from(10));
    ///
    /// assert!(engine.subtract().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(-5));
    /// ```
    pub fn subtract(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_dec(2)?;

        // Put result on stack
        let result = operands[0] - operands[1];
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Performs multiplication on the top two operands of the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top two values from the stack as [`Decimal`] numbers.
    /// - Computes the product of these values.
    /// - Pushes the result back onto the stack.
    /// - Returns [`EngineSignal::StackUpdated`] to indicate a successful operation.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if there are not enough operands on the stack or if the operands are invalid.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains fewer than two operands.
    /// - The operands are of an invalid type that cannot be converted to [`Decimal`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(5));
    /// engine.stack.push(Bucket::from(10));
    ///
    /// assert!(engine.multiply().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(50));
    /// ```
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

    /// Performs division on the top two operands of the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top two values from the stack as [`Decimal`] numbers.
    /// - Computes the quotient of these values.
    /// - Pushes the result back onto the stack.
    /// - Returns [`EngineSignal::StackUpdated`] to indicate a successful operation.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if there are not enough operands on the stack or if the operands are invalid.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains fewer than two operands.
    /// - The operands are of an invalid type that cannot be converted to [`Decimal`].
    /// - The denominator is zero
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(10));
    /// engine.stack.push(Bucket::from(5));
    ///
    /// assert!(engine.divide().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(2));
    /// ```
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

    /// Performs exponentiation on the top two operands of the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top two values from the stack as [`Decimal`] numbers.
    /// - Treats the item on the `top-1` of the stack as the base and the
    ///   item on the top of the stack as the exponent.
    /// - If the exponent is an integer, uses `checked_powd` from `rust_decimal`.
    /// - If the exponent is a decimal, converts both values to `f64` and uses `powf`.
    /// - Pushes the result back onto the stack.
    /// - Returns [`EngineSignal::StackUpdated`] to indicate a successful operation.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if there are not enough operands on the stack or if the operands are invalid.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains fewer than two operands.
    /// - Either operand is of an invalid type that cannot be converted to [`Decimal`].
    /// - Converting a value to `f64` fails.
    /// - Overflow occurs during exponentiation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(2));
    /// engine.stack.push(Bucket::from(3));
    ///
    /// assert!(engine.power().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(8));  // 2^3 = 8
    /// ```
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
                Some(value) => value
                    .to_f64()
                    .ok_or_else(|| format!("unable to convert {} to f64 in power", value))?,
                None => return Err("overflow when raising to a power".to_string()),
            }
        } else {
            // is a decimal
            let exponent = exponent
                .to_f64()
                .ok_or_else(|| format!("unable to convert {} to an f64 in power", exponent))?;
            base.to_f64()
                .ok_or_else(|| format!("unable to convert {} to an f64 in power", base))?
                .powf(exponent)
        };

        // Put result on stack
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Calculates the square root of the top operand on the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top value from the stack as a [`Decimal`] number.
    /// - Computes the square root of this value.
    /// - Pushes the result back onto the stack.
    /// - Returns [`EngineSignal::StackUpdated`] to indicate a successful operation.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if there are not enough operands on the stack or if the operands are invalid.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack is empty
    /// - The operand is of an invalid type that cannot be converted to [`Decimal`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(16));
    ///
    /// assert!(engine.sqrt().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(4));
    /// ```
    pub fn sqrt(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_dec(1)?;

        // Put result on stack
        let Some(result) = operands[0].sqrt() else {
            return Err("Error calculating sqrt".to_string());
        };
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Performs modulo on the top two operands of the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top two values from the stack as [`Decimal`] numbers.
    /// - Computes the euclidean modulo of these values.
    /// - Pushes the result back onto the stack.
    /// - Returns [`EngineSignal::StackUpdated`] to indicate a successful operation.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if there are not enough operands on the stack or if the operands are invalid.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains fewer than two operands.
    /// - The operands are of an invalid type that cannot be converted to [`Decimal`].
    /// - The denominator is zero
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(11));
    /// engine.stack.push(Bucket::from(2));
    ///
    /// assert!(engine.modulo().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(1));
    /// ```
    pub fn modulo(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        if operands[1] == 0.0 {
            return Err("cannot divide by zero".to_owned());
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

    /// Computes the sine of the top operand on the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top value from the stack.
    /// - Computes the sine of the value.
    /// - Pushes the result back onto the stack.
    /// - Returns [`EngineSignal::StackUpdated`] upon success.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if the stack is empty or the operand cannot be processed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains no operands.
    /// - The operand does not support the `sin` operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(0.0));
    ///
    /// assert!(engine.sin().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(0.0)); // sin(0) = 0
    /// ```
    pub fn sin(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let Some(result) = operands[0].sin() else {
            return Err("could not sin operand".to_string());
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Computes the cosine of the top operand on the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top value from the stack.
    /// - Computes the cosine of the value.
    /// - Pushes the result back onto the stack.
    /// - Returns [`EngineSignal::StackUpdated`] upon success.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if the stack is empty or the operand cannot be processed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains no operands.
    /// - The operand does not support the `cos` operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(0.0));
    ///
    /// assert!(engine.cos().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(1.0)); // cos(0) = 1
    /// ```
    pub fn cos(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let Some(result) = operands[0].cos() else {
            return Err("could not cos operand".to_string());
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Computes the tangent of the top operand on the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top value from the stack.
    /// - Computes the tangent of the value.
    /// - Pushes the result back onto the stack.
    /// - Returns [`EngineSignal::StackUpdated`] upon success.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if the stack is empty or the operand cannot be processed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains no operands.
    /// - The operand does not support the `tan` operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(0.0));
    ///
    /// assert!(engine.tan().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(0.0)); // tan(0) = 0
    /// ```
    pub fn tan(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_raw(1)?;
        // Put result on stack
        let Some(result) = operands[0].tan() else {
            return Err("could not tan operand".to_string());
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Computes the secant of the top operand on the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top value from the stack.
    /// - Computes the secant of the value.
    /// - Pushes the result back onto the stack.
    /// - Returns [`EngineSignal::StackUpdated`] upon success.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if the stack is empty or the operand cannot be processed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains no operands.
    /// - The operand does not support the `sec` operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(0.0));
    ///
    /// assert!(engine.sec().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(1.0)); // sec(0) = 1
    /// ```
    pub fn sec(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let Some(result) = operands[0].sec() else {
            return Err("could not sec operand".to_string());
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Computes the cosecant of the top operand on the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top value from the stack.
    /// - Computes the cosecant of the value.
    /// - Pushes the result back onto the stack.
    /// - Returns [`EngineSignal::StackUpdated`] upon success.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if the stack is empty or the operand cannot be processed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains no operands.
    /// - The operand does not support the `csc` operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(0.0));
    ///
    /// assert!(engine.csc().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::new_undefined()); // csc(0) = undefined
    /// ```
    pub fn csc(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let Some(result) = operands[0].csc() else {
            return Err("could not csc operand".to_string());
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Computes the cotangent of the top operand on the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top value from the stack.
    /// - Computes the cotangent of the value.
    /// - Pushes the result back onto the stack.
    /// - Returns [`EngineSignal::StackUpdated`] upon success.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if the stack is empty or the operand cannot be processed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains no operands.
    /// - The operand does not support the `cot` operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(0.0));
    ///
    /// assert!(engine.csc().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::new_undefined()); // cot(0) = undefined
    /// ```
    pub fn cot(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_raw(1)?;

        // Put result on stack
        let Some(result) = operands[0].cot() else {
            return Err("could not sine operand".to_string());
        };
        let _ = self.add_item_to_stack(result);
        Ok(EngineSignal::StackUpdated)
    }

    /// Computes the arcsine (inverse sine) of the top operand on the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top value from the stack.
    /// - Computes the arcsine of the value (in radians).
    /// - Pushes the result back onto the stack.
    /// - Returns `EngineSignal::StackUpdated` upon success.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if the stack is empty or the operand cannot be processed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains no operands.
    /// - The operand is not a valid input for the arcsine function (e.g., out of the range [-1, 1]).
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::{Bucket, ConstantTypes};
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(1.0));
    ///
    /// assert!(engine.asin().is_ok());
    /// // asin(1) = HalfPi
    /// assert_eq!(engine.stack.last().unwrap().value, Bucket::from_constant(ConstantTypes::HalfPi).value);
    /// ```
    pub fn asin(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].asin().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Computes the arccosine (inverse cosine) of the top operand on the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top value from the stack.
    /// - Computes the arccosine of the value (in radians).
    /// - Pushes the result back onto the stack.
    /// - Returns `EngineSignal::StackUpdated` upon success.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if the stack is empty or the operand cannot be processed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains no operands.
    /// - The operand is not a valid input for the arcsine function (e.g., out of the range [-1, 1]).
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::{Bucket, ConstantTypes};
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(-1.0));
    ///
    /// assert!(engine.acos().is_ok());
    /// // acos(-1) = Pi
    /// assert_eq!(engine.stack.last().unwrap().value, Bucket::from_constant(ConstantTypes::Pi).value);
    /// ```
    pub fn acos(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].acos().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Computes the arctangent (inverse tangent) of the top operand on the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top value from the stack.
    /// - Computes the arctangent of the value (in radians).
    /// - Pushes the result back onto the stack.
    /// - Returns `EngineSignal::StackUpdated` upon success.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if the stack is empty or the operand cannot be processed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains no operands.
    /// - The operand is not a valid input for the arcsine function (e.g., out of the range [-sqrt(3), sqrt(3)]).
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::{Bucket, ConstantTypes};
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(1.0));
    ///
    /// assert!(engine.atan().is_ok());
    /// // atan(1) = pi/4
    /// assert_eq!(engine.stack.last().unwrap().value, Bucket::from_constant(ConstantTypes::QuarterPi).value);
    /// ```
    pub fn atan(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].atan().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Changes the sign (negates) of the top operand on the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top value from the stack.
    /// - Negates the value (multiplies by -1).
    /// - Pushes the negated result back onto the stack.
    /// - Returns `EngineSignal::StackUpdated` upon success.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if the stack is empty or the operand cannot be processed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains no operands.
    /// - The operand cannot be processed (e.g., invalid type).
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(10.0));
    ///
    /// assert!(engine.chs().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(-10.0));
    /// ```
    pub fn chs(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let result = operands[0] * -1.0;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Computes the base-10 logarithm of the top operand on the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top value from the stack.
    /// - Computes the base-10 logarithm (log10) of the value.
    /// - If the value is 0 or negative, it returns an error.
    /// - Pushes the result of the logarithm back onto the stack.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if the operand is less than or equal to 0, or if the stack is empty.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains no operands.
    /// - The operand is less than or equal to zero (logarithm of 0 or negative number is undefined).
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(100.0));
    ///
    /// assert!(engine.log().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(2.0)); // log10(100) = 2
    /// ```
    pub fn log(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_dec(1)?;

        // Put result on stack
        let Some(result) = operands[0].checked_log10() else {
            return Err("cannot take log10 of 0 or negative numbers".to_string());
        };
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Computes the logarithm of a number with a specified base using the change of base formula.
    ///
    /// The change of base formula is:
    ///
    /// ```text
    /// log_b(a) = (log_d(a)) / (log_d(b))
    /// ```
    ///
    /// where:
    /// - `a` is the number whose logarithm is to be calculated.
    /// - `b` is the base of the logarithm.
    ///
    /// # Behavior
    ///
    /// - Retrieves two operands from the stack: the number `a` and the base `b`.
    /// - Uses the change of base formula to compute the logarithm of `a` with base `b`.
    /// - If either operand is 0 or negative, or if division by zero occurs, it returns an error.
    /// - Pushes the result back onto the stack.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if any of the following errors occur:
    ///   - Either operand is 0 or negative.
    ///   - Division by zero occurs.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Either operand is less than or equal to 0.
    /// - Division by zero occurs during the calculation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(100.0));  // 'a' value
    /// engine.stack.push(Bucket::from(10.0));   // 'b' value (base)
    ///
    /// assert!(engine.blog().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(2.0)); // log_10(100) = 2
    /// ```
    pub fn blog(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_dec(2)?;

        // change of base formula is defined as follows:
        // log_b(a) = (log_d(a))/(log_d(b))

        let Some(top_log) = operands[0].checked_log10() else {
            return Err("cannot take log of 0 or negative numbers".to_string());
        };
        let Some(bottom_log) = operands[1].checked_log10() else {
            return Err("cannot take log with base of 0 or negative numbers".to_string());
        };

        let Some(result) = top_log.checked_div(bottom_log) else {
            return Err("cannot divide by zero".to_string());
        };

        // Put result on stack
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Computes the natural logarithm (ln) of a number.
    ///
    /// The natural logarithm is the logarithm to the base of Euler's number (e).
    /// It is defined for positive real numbers.
    ///
    /// # Behavior
    ///
    /// - Retrieves one operand from the stack (the number `a`).
    /// - Computes the natural logarithm (ln) of the operand.
    /// - If the operand is 0 or negative, returns an error.
    /// - Pushes the result back onto the stack.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if the operand is 0 or negative.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The operand is less than or equal to 0, as the natural logarithm is only defined for positive real numbers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::{Bucket, ConstantTypes};
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from_constant(ConstantTypes::E));  // Euler's number
    ///
    /// assert!(engine.ln().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(1.0)); // ln(e) = 1
    /// ```
    pub fn ln(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_dec(1)?;

        // Put result on stack
        let Some(result) = operands[0].checked_ln() else {
            return Err("cannot take log10 of 0 or negative numbers".to_string());
        };
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Computes the absolute value of the top operand on the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top value from the stack.
    /// - Takes the absolute value
    /// - Pushes the negated result back onto the stack.
    /// - Returns `EngineSignal::StackUpdated` upon success.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - `Err(String)` if the stack is empty or the operand cannot be processed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack contains no operands.
    /// - The operand cannot be processed (e.g., invalid type).
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(-10.0));
    ///
    /// assert!(engine.abs().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(10.0));
    /// ```
    pub fn abs(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].abs().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Compares two operands for equality.
    ///
    /// This function compares the top two operands on the stack to check if they are equal.
    ///
    /// # Behavior
    ///
    /// - Retrieves two operands from the stack (operands `a` and `b`).
    /// - Compares the operands for equality (`a == b`).
    /// - If the operands are equal, pushes `1` (as a `u32`) onto the stack; otherwise, pushes `0`.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - The result is `1` if the operands are equal, otherwise `0`.
    ///
    /// # Errors
    ///
    /// This function does not explicitly handle errors related to operand types.
    /// It assumes that the operands are of compatible types (in this case, `f64`).
    /// If incompatible types are used, an error will be returned by the `get_operands_as_f` function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(5.0));
    /// engine.stack.push(Bucket::from(5.0));
    ///
    /// assert!(engine.equal().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(1)); // 5.0 == 5.0, so result is 1
    /// ```
    pub fn equal(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        // TODO: maybe make this work with strings
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] == operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Compares two operands to check if the first is greater than the second.
    ///
    /// This function compares the top two operands on the stack to check if the first operand
    /// is greater than the second operand.
    ///
    /// # Behavior
    ///
    /// - Retrieves two operands from the stack (operands `a` and `b`).
    /// - Compares the operands to check if `a > b`.
    /// - If the first operand is greater, pushes `1` (as a `u32`) onto the stack; otherwise, pushes `0`.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - The result is `1` if the first operand is greater than the second, otherwise `0`.
    ///
    /// # Errors
    ///
    /// This function does not explicitly handle errors related to operand types.
    /// It assumes that the operands are of compatible types (in this case, `f64`).
    /// If incompatible types are used, an error will be returned by the `get_operands_as_f` function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(10.0));
    /// engine.stack.push(Bucket::from(5.0));
    ///
    /// assert!(engine.gt().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(1)); // 10.0 > 5.0, so result is 1
    /// ```
    pub fn gt(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] > operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Compares two operands to check if the first is less than the second.
    ///
    /// This function compares the top two operands on the stack to check if the first operand
    /// is less than the second operand.
    ///
    /// # Behavior
    ///
    /// - Retrieves two operands from the stack (operands `a` and `b`).
    /// - Compares the operands to check if `a < b`.
    /// - If the first operand is less, pushes `1` (as a `u32`) onto the stack; otherwise, pushes `0`.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - The result is `1` if the first operand is greater than the second, otherwise `0`.
    ///
    /// # Errors
    ///
    /// This function does not explicitly handle errors related to operand types.
    /// It assumes that the operands are of compatible types (in this case, `f64`).
    /// If incompatible types are used, an error will be returned by the `get_operands_as_f` function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(5.0));
    /// engine.stack.push(Bucket::from(10.0));
    ///
    /// assert!(engine.lt().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(1)); // 5.0 < 10.0, so result is 1
    /// ```
    pub fn lt(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] < operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Compares two operands to check if the first is greater than or equal to the second.
    ///
    /// This function compares the top two operands on the stack to check if the first operand
    /// is greater than or equal to the second operand.
    ///
    /// # Behavior
    ///
    /// - Retrieves two operands from the stack (operands `a` and `b`).
    /// - Compares the operands to check if `a >= b`.
    /// - If the first operand is greater or equal, pushes `1` (as a `u32`) onto the stack; otherwise, pushes `0`.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - The result is `1` if the first operand is greater than or equal to the second, otherwise `0`.
    ///
    /// # Errors
    ///
    /// This function does not explicitly handle errors related to operand types.
    /// It assumes that the operands are of compatible types (in this case, `f64`).
    /// If incompatible types are used, an error will be returned by the `get_operands_as_f` function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(10.0));
    /// engine.stack.push(Bucket::from(9.0));
    ///
    /// assert!(engine.geq().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(1)); // 10.0 >= 9.0, so result is 1
    /// ```
    pub fn geq(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] >= operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Compares two operands to check if the first is less than or equal to the second.
    ///
    /// This function compares the top two operands on the stack to check if the first operand
    /// is less than or equal to the second operand.
    ///
    /// # Behavior
    ///
    /// - Retrieves two operands from the stack (operands `a` and `b`).
    /// - Compares the operands to check if `a <= b`.
    /// - If the first operand is less or equal, pushes `1` (as a `u32`) onto the stack; otherwise, pushes `0`.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - The result is `1` if the first operand is less than or equal to the second, otherwise `0`.
    ///
    /// # Errors
    ///
    /// This function does not explicitly handle errors related to operand types.
    /// It assumes that the operands are of compatible types (in this case, `f64`).
    /// If incompatible types are used, an error will be returned by the `get_operands_as_f` function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(9.0));
    /// engine.stack.push(Bucket::from(10.0));
    ///
    /// assert!(engine.leq().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(1)); // 9.0 <= 10.0, so result is 1
    /// ```
    pub fn leq(&mut self) -> Result<EngineSignal, String> {
        // Get operands
        let operands = self.get_operands_as_f(2)?;

        // Put result on stack
        let result = (operands[0] <= operands[1]) as u32;
        let _ = self.add_item_to_stack(result.into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Rounds a number to the nearest integer.
    ///
    /// This function takes the top operand from the stack and rounds it to the nearest integer.
    /// The result is then pushed back onto the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves one operand from the stack.
    /// - Rounds the operand to the nearest integer using Rust's [`f64::round`] method.
    /// - Pushes the rounded result back onto the stack.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful.
    /// - The result is the rounded value of the operand.
    ///
    /// # Errors
    ///
    /// This function does not explicitly handle errors related to operand types. It assumes that
    /// the operand is of type `f64`. If the operand is of an incompatible type, an error will be
    /// returned by the `get_operands_as_f` function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(3.7));
    ///
    /// assert!(engine.round().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(4.0)); // Rounded from 3.7 to 4.0
    /// ```
    pub fn round(&mut self) -> Result<EngineSignal, String> {
        // Get operand
        let operands = self.get_operands_as_f(1)?;

        // Put result on stack
        let _ = self.add_item_to_stack(operands[0].round().into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Inverts the top operand on the stack.
    ///
    /// This function takes the top operand from the stack and computes its multiplicative inverse (1 / operand).
    /// If the operand is zero, an error is returned since division by zero is undefined.
    /// The result is then pushed back onto the stack.
    ///
    /// # Behavior
    ///
    /// - Retrieves one operand from the stack.
    /// - Checks if the operand is zero. If it is, returns an error (division by zero).
    /// - Computes the inverse of the operand (1 / operand) if it is non-zero.
    /// - Pushes the result back onto the stack.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` if the operation is successful and the result is pushed onto the stack.
    /// - `Err(String)` if the operand is zero, indicating division by zero.
    ///
    /// # Errors
    ///
    /// - Returns an error if the operand is zero, as division by zero is not allowed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(5.0));
    ///
    /// assert!(engine.invert().is_ok());
    /// assert_eq!(engine.stack.last().unwrap(), &Bucket::from(0.2)); // Inverted from 5.0 to 0.2
    ///
    /// engine.stack.push(Bucket::from(0.0));
    /// assert!(engine.invert().is_err()); // Division by zero error
    /// ```
    pub fn invert(&mut self) -> Result<EngineSignal, String> {
        // Get operand
        let operands = self.get_operands_as_f(1)?;

        if operands[0] == 0.0 {
            return Err("cannot divide by zero".to_string());
        }

        // Put result on stack
        let _ = self.add_item_to_stack((1_f64 / operands[0]).into());
        Ok(EngineSignal::StackUpdated)
    }

    /// Removes the top item from the stack.
    ///
    /// This function pops the last item from the stack. If the stack is not empty, the item is removed.
    /// If the stack is empty, the function does nothing but still returns [`EngineSignal::StackUpdated`].
    ///
    /// # Behavior
    ///
    /// - Removes the top item from the stack.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` when the operation is successfully completed and the stack is updated.
    ///
    /// # Errors
    ///
    /// This function doesn't return an error variant, and is only written this way to be
    /// compatible with the other engine functions.
    ///
    /// # Notes
    ///
    /// - If the stack is empty, the function will simply return without making changes, but still indicate that the stack has been updated.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(10.0));
    ///
    /// assert!(engine.drop().is_ok());
    /// assert!(engine.stack.is_empty()); // Stack is empty after dropping the item
    /// ```
    pub fn drop(&mut self) -> Result<EngineSignal, String> {
        // Remove last item from stack
        self.stack.pop();
        Ok(EngineSignal::StackUpdated)
    }

    /// Swaps the top two items on the stack.
    ///
    /// This function takes the last two values from the stack, removes them, and then places them back in reverse order.
    ///
    /// # Behavior
    ///
    /// - Retrieves the top two items from the stack.
    /// - Swaps their positions on the stack.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` when the operation is successfully completed and the stack is updated.
    ///
    /// # Errors
    ///
    /// This function doesn't return an error variant, and is only written this way to be
    /// compatible with the other engine functions.
    ///
    /// # Notes
    ///
    /// - If the stack has fewer than two items, the function will return an error indicating that not enough operands are present.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(5.0));
    /// engine.stack.push(Bucket::from(10.0));
    ///
    /// assert!(engine.swap().is_ok());
    /// assert_eq!(engine.stack[0], Bucket::from(10.0)); // Top item after swap
    /// assert_eq!(engine.stack[1], Bucket::from(5.0));  // Second item after swap
    /// ```
    pub fn swap(&mut self) -> Result<EngineSignal, String> {
        // Get last two values from stack
        let operands = self.get_operands_raw(2)?;

        // Insert in reverse order
        let _ = self.add_item_to_stack(operands[1].clone());
        let _ = self.add_item_to_stack(operands[0].clone());
        Ok(EngineSignal::StackUpdated)
    }

    /// Duplicates the last item on the stack.
    ///
    /// This function takes the top item from the stack and pushes it onto the stack twice, duplicating the top value.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` when the operation is successfully completed and the stack is updated.
    ///
    /// # Errors
    ///
    /// This function doesn't return an error variant, and is only written this way to be
    /// compatible with the other engine functions.
    ///
    /// # Notes
    ///
    /// - If the stack is empty, the function will return an error indicating that no operand is available for duplication.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(5.0));
    ///
    /// assert!(engine.dup().is_ok());
    /// assert_eq!(engine.stack[0], Bucket::from(5.0)); // Top item after duplication
    /// assert_eq!(engine.stack[1], Bucket::from(5.0)); // Duplicate item
    /// ```
    pub fn dup(&mut self) -> Result<EngineSignal, String> {
        // Get the last value from the stack
        let operands = self.get_operands_raw(1)?;

        // Insert twice
        let _ = self.add_item_to_stack(operands[0].clone());
        let _ = self.add_item_to_stack(operands[0].clone());
        Ok(EngineSignal::StackUpdated)
    }

    /// Rolls the stack down, rotating the elements to the right.
    ///
    /// This function moves the topmost item to the bottom of the stack by rotating the stack right by one position.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` when the operation is successfully completed and the stack is updated.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack is empty.
    ///
    /// # Notes
    ///
    /// - If the stack is empty, the function will return an error indicating that the operation cannot be performed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(1.0));
    /// engine.stack.push(Bucket::from(2.0));
    /// engine.stack.push(Bucket::from(3.0));
    ///
    /// assert!(engine.roll_down().is_ok());
    /// assert_eq!(engine.stack[0], Bucket::from(3.0)); // Top item after roll
    /// assert_eq!(engine.stack[2], Bucket::from(2.0)); // Last item moved to the bottom
    /// ```
    pub fn roll_down(&mut self) -> Result<EngineSignal, String> {
        if self.stack.is_empty() {
            Err(String::from("Cannot roll empty stack"))
        } else {
            // Rotate stack right
            self.stack.rotate_right(1);
            Ok(EngineSignal::StackUpdated)
        }
    }

    /// Rolls the stack up, rotating the elements to the left.
    ///
    /// This function moves the bottommost item to the top of the stack by rotating the stack left by one position.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` when the operation is successfully completed and the stack is updated.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The stack is empty.
    ///
    /// # Notes
    ///
    /// - If the stack is empty, the function will return an error indicating that the operation cannot be performed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(1.0));
    /// engine.stack.push(Bucket::from(2.0));
    /// engine.stack.push(Bucket::from(3.0));
    ///
    /// assert!(engine.roll_up().is_ok());
    /// assert_eq!(engine.stack[2], Bucket::from(1.0)); // Bottom item moved to the top
    /// assert_eq!(engine.stack[0], Bucket::from(2.0));
    /// ```
    pub fn roll_up(&mut self) -> Result<EngineSignal, String> {
        if self.stack.is_empty() {
            Err(String::from("Cannot roll empty stack"))
        } else {
            // Rotate stack left
            self.stack.rotate_left(1);
            Ok(EngineSignal::StackUpdated)
        }
    }

    /// Stores a value in a variable.
    ///
    /// This function stores the topmost value from the stack into a variable, validating that the variable name follows a valid identifier pattern.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` when the operation is successfully completed and the stack is updated.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The variable name is not a valid ID
    ///
    /// # Notes
    ///
    /// - If the second operand is not a valid variable name, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(5.0));
    /// engine.stack.push(Bucket::from("var1"));
    ///
    /// assert!(engine.store().is_ok());
    /// // Variable "var1" should now store the value 5.0
    /// assert!(engine.variables.get("var1").is_some());
    /// ```
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

    /// Deletes a variable from memory.
    ///
    /// This function removes a variable from the internal variable store if it exists. It only works if the variable is a valid identifier.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` when the operation is successfully completed and the stack is updated.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The variable name is not a valid ID
    /// - The variable is not set
    ///
    /// # Notes
    ///
    /// - If the variable does not exist or is not a valid identifier, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(5.0));
    /// engine.stack.push(Bucket::from("var1"));
    /// engine.store(); // Store a value in a variable
    /// assert!(engine.variables.get("var1").is_some());
    ///
    /// engine.stack.push(Bucket::from("var1"));
    /// assert!(engine.purge().is_ok());
    /// // Variable should now be removed
    /// assert!(engine.variables.get("var1").is_none());
    /// ```
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

    /// Stores a value in a variable, with inverted argument order. Useful for how variable
    /// assignments are parsed in algebraic mode.
    ///
    /// This function swaps the top two values on the stack and then calls the `store` function to store the value in the variable.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` when the operation is successfully completed and the stack is updated.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The variable name is not a valid ID
    ///
    /// # Notes
    ///
    /// - This function uses `swap()` and will return an error if swapping fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from("var1"));
    /// engine.stack.push(Bucket::from(5.0));
    ///
    /// assert!(engine.invstore().is_ok());
    /// // Variable "var1" should now store the value 5.0
    /// assert!(engine.variables.get("var1").is_some());
    /// ```
    pub fn invstore(&mut self) -> Result<EngineSignal, String> {
        self.swap()?;
        self.store()
    }

    /// Clears the entire stack.
    ///
    /// This function empties the stack, removing all items stored in it.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` when the stack has been successfully cleared.
    ///
    /// # Errors
    ///
    /// This function doesn't return an error variant, and is only written this way to be
    /// compatible with the other engine functions.
    ///
    /// # Notes
    ///
    /// - This function does not return an error; the stack is simply cleared.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.stack.push(Bucket::from(5.0));
    /// engine.stack.push(Bucket::from(10.0));
    ///
    /// assert!(engine.clear().is_ok());
    /// assert!(engine.stack.is_empty()); // Stack is now empty
    /// ```
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

    /// Undoes the last operation by reverting the stack and variables to their previous state.
    ///
    /// This function reverts the stack and variables to the state they were in at the time of the last operation.
    /// If no operations have been performed or the undo history has been exhausted, an error is returned.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` when the undo is successfully performed and the stack and variables are updated.
    /// - `Err(String)` if there is no operation to undo.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - You try to undo further than the history allows
    ///
    /// # Notes
    ///
    /// - The undo history is stored, and each undo operation increments a pointer to track the state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    /// use squiid_engine;
    ///
    /// let mut engine = Engine::new();
    /// // after each command, we must push a copy of the stack to the engine history
    /// let _ = squiid_engine::handle_data(&mut engine, "1");
    /// let _ = squiid_engine::handle_data(&mut engine, "2");
    /// let _ = squiid_engine::handle_data(&mut engine, "test");
    ///
    /// // test undo of adding something to the stack
    /// let _ = engine.undo();
    /// assert_eq!(engine.stack, vec![Bucket::from(1), Bucket::from(2),]);
    /// ```
    pub fn undo(&mut self) -> Result<EngineSignal, String> {
        if self.undo_state_pointer < self.undo_history.len() as u8 {
            if self.undo_state_pointer == 0 {
                // add current stack and variables to history and increment pointer by 1
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

    /// Redoes the last undone operation by restoring the stack and variables.
    ///
    /// This function restores the stack and variables to the state they were in at the time of the last undone operation.
    /// If no operations have been undone or the redo history is exhausted, an error is returned.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::StackUpdated)` when the redo is successfully performed and the stack and variables are updated.
    /// - `Err(String)` if there are no operations to redo.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - You try to redo further than the number of undone operations
    ///
    /// # Notes
    ///
    /// - The redo history is tracked using the undo state pointer to determine the next state to restore.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    /// use squiid_engine;
    ///
    /// let mut engine = Engine::new();
    /// // after each command, we must push a copy of the stack to the engine history
    /// let _ = squiid_engine::handle_data(&mut engine, "1");
    /// let _ = squiid_engine::handle_data(&mut engine, "2");
    /// let _ = squiid_engine::handle_data(&mut engine, "test");
    ///
    /// // test undo of adding something to the stack
    /// let _ = engine.undo();
    /// assert_eq!(engine.stack, vec![Bucket::from(1), Bucket::from(2),]);
    ///
    /// // test redo
    /// let _ = engine.redo();
    /// assert_eq!(engine.stack, vec![Bucket::from(1), Bucket::from(2), Bucket::from("test")]);
    /// ```
    pub fn redo(&mut self) -> Result<EngineSignal, String> {
        if self.undo_state_pointer > 1 {
            self.undo_state_pointer -= 1;
            self.update_engine_from_history();
            Ok(EngineSignal::StackUpdated)
        } else {
            Err(String::from("Cannot redo further"))
        }
    }

    /// Terminates the engine and signals the system to quit.
    ///
    /// This function sends a quit signal to the engine, indicating that it should stop processing and exit.
    ///
    /// # Returns
    ///
    /// - `Ok(EngineSignal::Quit)` when the quit signal is sent.
    ///
    /// # Errors
    ///
    /// This function doesn't return an error variant, and is only written this way to be
    /// compatible with the other engine functions.
    ///
    /// # Notes
    ///
    /// - This function does not affect the state of the stack or variables but instead signals the engine to stop execution.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squiid_engine::bucket::Bucket;
    /// use squiid_engine::engine::Engine;
    ///
    /// let mut engine = Engine::new();
    ///
    /// assert!(engine.quit().is_ok()); // The engine is now quitting
    /// ```
    pub fn quit(&mut self) -> Result<EngineSignal, String> {
        Ok(EngineSignal::Quit)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
