use std::num::ParseFloatError;

use thiserror::Error;

/// Define a new error enum which has an operation-specific error, as well as all variants from [`OperandError`]
macro_rules! define_operation_error {
    ($name:ident, $desc:literal) => {
        #[derive(thiserror::Error, Debug)]
        pub enum $name {
            #[error($desc)]
            $name,
            #[error(transparent)]
            OperandError(#[from] OperandError),
        }
    };

    ($name:ident, $desc:literal $(, $t:ty)*) => {
        #[derive(thiserror::Error, Debug)]
        pub enum $name {
            #[error($desc)]
            $name($($t),*),
            #[error(transparent)]
            OperandError(#[from] OperandError),
        }
    };
}

/// Generates an aggregate engine error out of all the errors passed to it
macro_rules! generate_engine_error {
    ($($name:ident),*) => {
        #[derive(thiserror::Error, Debug)]
        pub enum EngineError {
            $(
                #[error(transparent)]
                $name(#[from] $name),
            )*
        }
    };
}

generate_engine_error!(
    SqrtError,
    SinError,
    CosError,
    TanError,
    SecError,
    CscError,
    CotError,
    UndefinedVariableReferenceError,
    OperandError,
    EmptyStackError,
    DivisionError,
    PowerError,
    LogDomainError,
    BaseLogError,
    EmptyStackRollError,
    StoreVariableInvalidIDError,
    PurgeError,
    UndoLimitError,
    RedoLimitError
);

define_operation_error!(SqrtError, "Error calculating square root");

define_operation_error!(SinError, "Could not sine operand");

define_operation_error!(CosError, "Could not cosine operand");

define_operation_error!(TanError, "Could not tangent operand");

define_operation_error!(SecError, "Could not secant operand");

define_operation_error!(CscError, "Could not cosecant operand");

define_operation_error!(CotError, "Could not cotangent operand");

define_operation_error!(LogDomainError, "Cannot take log of 0 or negative numbers");

define_operation_error!(
    StoreVariableInvalidIDError,
    "Cannot store in non-variable object `{0}`",
    String
);

#[derive(Error, Debug)]
pub enum DivisionError {
    #[error(transparent)]
    ZeroDivisionError(#[from] ZeroDivisionError),
    #[error(transparent)]
    OperandError(#[from] OperandError),
}

#[derive(Error, Debug)]
pub enum PowerError {
    #[error("Unable to convert {0} to f64")]
    FloatConversionError(String),
    #[error("Overflow when raising to a power")]
    PowerOverflowError,
    #[error(transparent)]
    OperandError(#[from] OperandError),
}

#[derive(Error, Debug)]
pub enum PurgeError {
    #[error("Variable `{0}` does not exist")]
    NonExistantVariableError(String),
    #[error("Cannot delete non-variable object `{0}`")]
    DeleteVariableInvalidIDError(String),
    #[error(transparent)]
    OperandError(#[from] OperandError),
}

#[derive(Error, Debug)]
pub enum BaseLogError {
    #[error("Cannot take log with base of 0 or negative numbers")]
    LogBaseDomainError,
    #[error(transparent)]
    ZeroDivisionError(#[from] ZeroDivisionError),
    #[error("Cannot take log of 0 or negative numbers")]
    LogDomainError,
    #[error(transparent)]
    OperandError(#[from] OperandError),
}

#[derive(Error, Debug)]
pub enum OperandError {
    #[error(transparent)]
    InvalidOperandTypeError(#[from] InvalidOperandTypeError),
    #[error(transparent)]
    PopFailureError(#[from] PopFailureError),
    #[error(transparent)]
    FloatParseError(#[from] FloatParseError),
    #[error(transparent)]
    NotEnoughItemsError(#[from] NotEnoughItemsError),
    #[error(transparent)]
    MissingValueError(#[from] MissingValueError),
    #[error(transparent)]
    DecimalReprError(#[from] DecimalReprError),
}

#[derive(Error, Debug)]
#[error("reference to undefined variable: {0}")]
pub struct UndefinedVariableReferenceError(pub String);

#[derive(Error, Debug)]
#[error("The operation cannot be performed on these operands")]
pub struct InvalidOperandTypeError;

#[derive(Error, Debug)]
#[error("Not enough items on stack for operation")]
pub struct NotEnoughItemsError;

#[derive(Error, Debug)]
#[error("Failed to pop operand")]
pub struct PopFailureError;

#[derive(Error, Debug)]
#[error("Operand value is missing")]
pub struct MissingValueError;

#[derive(Error, Debug)]
#[error("Failed to parse operand as f64: {0}")]
pub struct FloatParseError(#[from] pub ParseFloatError);

#[derive(Error, Debug)]
#[error("Stack is empty")]
pub struct EmptyStackError;

#[derive(Error, Debug)]
#[error("Cannot divide by 0")]
pub struct ZeroDivisionError;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct DecimalReprError(#[from] pub rust_decimal::Error);

#[derive(Error, Debug)]
#[error("Cannot roll empty stack")]
pub struct EmptyStackRollError;

#[derive(Error, Debug)]
#[error("Cannot undo further")]
pub struct UndoLimitError;

#[derive(Error, Debug)]
#[error("Cannot redo further")]
pub struct RedoLimitError;
