use std::num::ParseFloatError;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum OperandError {
    #[error("reference to undefined variable: {0}")]
    UndefinedVariableReferenceError(String),
    #[error("The operation cannot be performed on these operands")]
    InvalidOperandType,
    #[error("Not enough items on stack for operation")]
    NotEnoughItems,
    #[error("Failed to pop operand")]
    PopFailure,
    #[error("Operand value is missing")]
    MissingValue,
    #[error("Failed to parse operand as f64: {0}")]
    FloatParseError(#[from] ParseFloatError),
    #[error("Stack is empty")]
    EmptyStackError,
    #[error("Cannot divide by 0")]
    ZeroDivisionError,
    #[error(transparent)]
    DecimalReprError(#[from] rust_decimal::Error),
    #[error("Unable to convert {0} to f64")]
    FloatConversionError(String),
    #[error("Overflow when raising to a power")]
    PowerOverflowError,
    #[error(transparent)]
    OperationError(#[from] OperationError),
}

#[derive(Error, Debug, PartialEq)]
pub enum OperationError {
    #[error("Error calculating square root")]
    Sqrt,
    #[error("Could not sine operand")]
    Sin,
    #[error("Could not cosine operand")]
    Cos,
    #[error("Could not tangent operand")]
    Tan,
    #[error("Could not secant operand")]
    Sec,
    #[error("Could not cosecant operand")]
    Csc,
    #[error("Could not cotangent operand")]
    Cot,
    #[error("Cannot take log of 0 or negative numbers")]
    LogDomain,
    #[error("Cannot take log with base of 0 or negative numbers")]
    LogBaseDomain,
    #[error("Cannot roll empty stack")]
    EmptyStackRoll,
    #[error("Cannot store in non-variable object `{0}`")]
    StoreVariableInvalidID(String),
    #[error("Variable `{0}` does not exist")]
    NonExistantVariable(String),
    #[error("Cannot delete non-variable object `{0}`")]
    DeleteVariableInvalidID(String),
    #[error("Cannot undo further")]
    UndoLimit,
    #[error("Cannot redo further")]
    RedoLimit,
}
