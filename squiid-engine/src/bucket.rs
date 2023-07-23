// items on the stack are called Buckets

use std::{f64::consts, collections::HashMap};

use rust_decimal::{prelude::FromPrimitive, Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use serde::{de::Visitor, Deserialize, Serialize};

/// Types of constants
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ConstantTypes {
    PI,
    HalfPI,
    ThirdPI,
    QuarterPI,
    SixthPI,
    EighthPI,
    TwoPI,
    E,
    TAU,
    C,
    G,
    PHI,
}

// Define the exposed constants
pub trait ExposedConstant {
    fn name(&self) -> &'static str;
}

impl ExposedConstant for ConstantTypes {
    fn name(&self) -> &'static str {
        match self {
            ConstantTypes::PI => "#pi",
            ConstantTypes::E => "#e",
            ConstantTypes::TAU => "#tau",
            ConstantTypes::C => "#c",
            ConstantTypes::G => "#G",
            ConstantTypes::PHI => "#phi",
            ConstantTypes::HalfPI => "#halfpi",
            ConstantTypes::ThirdPI => "#thirdpi",
            ConstantTypes::QuarterPI => "#quarterpi",
            ConstantTypes::SixthPI => "#sixthpi",
            ConstantTypes::EighthPI => "#eighthpi",
            ConstantTypes::TwoPI => "#twopi",
        }
    }
}

// TODO: extrapolate constants things into constants file
/// Build a hashmap of exposed constants
pub fn build_exposed_constants() -> HashMap<&'static str, ConstantTypes> {
    let mut exposed_constants = HashMap::new();

    // Add each constant to the hashmap
    exposed_constants.insert(ConstantTypes::PI.name(), ConstantTypes::PI);
    exposed_constants.insert(ConstantTypes::E.name(), ConstantTypes::E);
    exposed_constants.insert(ConstantTypes::TAU.name(), ConstantTypes::TAU);
    exposed_constants.insert(ConstantTypes::C.name(), ConstantTypes::C);
    exposed_constants.insert(ConstantTypes::G.name(), ConstantTypes::G);
    exposed_constants.insert(ConstantTypes::PHI.name(), ConstantTypes::PHI);

    exposed_constants
}



/// Types of Buckets
#[derive(Debug, Clone, PartialEq)]
pub enum BucketTypes {
    Float,
    String,
    Constant(ConstantTypes),
    // TODO: should undefined error out? in trig and stuff
    Undefined,
}

/// Bucket contains the items that can be on the stack
#[derive(Debug, Clone, PartialEq)]
pub struct Bucket {
    /// Bucket value. Will be None when undefined
    pub value: Option<String>,
    /// The type of the Bucket
    pub bucket_type: BucketTypes,
}

impl Bucket {
    /// Create a new undefined Bucket
    pub fn new_undefined() -> Self {
        Self {
            value: None,
            bucket_type: BucketTypes::Undefined,
        }
    }

    /// Create a Bucket from a constant
    pub fn from_constant(constant_type: ConstantTypes) -> Self {
        let value = match constant_type {
            ConstantTypes::PI => consts::PI,
            ConstantTypes::HalfPI => consts::FRAC_PI_2,
            ConstantTypes::ThirdPI => consts::FRAC_PI_3,
            ConstantTypes::QuarterPI => consts::FRAC_PI_4,
            ConstantTypes::SixthPI => consts::FRAC_PI_6,
            ConstantTypes::EighthPI => consts::FRAC_PI_8,
            ConstantTypes::TwoPI => consts::PI * 2.0,
            ConstantTypes::E => consts::E,
            ConstantTypes::TAU => consts::TAU,
            ConstantTypes::C => 299792458_f64,
            ConstantTypes::G => 6.67430 * 10_f64.powf(-11_f64),
            ConstantTypes::PHI => 1.6180339887498948482045868343656381,
        }
        .to_string();

        Bucket {
            value: Some(value),
            bucket_type: BucketTypes::Constant(constant_type),
        }
    }

    /// Sine
    pub fn sin(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                ConstantTypes::E | ConstantTypes::TAU | ConstantTypes::C | ConstantTypes::G | ConstantTypes::PHI => {
                    Some(Self::from(
                        self.value.clone()?.parse::<f64>().unwrap().sin(),
                    ))
                }
                ConstantTypes::PI => Some(Self::from(0)),
                ConstantTypes::TwoPI => Some(Self::from(0)),
                ConstantTypes::HalfPI => Some(Self::from(1)),
                ConstantTypes::QuarterPI => Some(Self::from(consts::FRAC_1_SQRT_2)),
                ConstantTypes::EighthPI => Some(Self::from(consts::FRAC_PI_8.sin())),
                ConstantTypes::SixthPI => Some(Self::from(0.5)),
                ConstantTypes::ThirdPI => Some(Self::from(consts::FRAC_PI_3.sin())),
            },
            BucketTypes::Float => Some(Self::from(
                Decimal::from_f64(self.value.clone()?.parse::<f64>().unwrap())?.checked_sin()?,
            )),
            BucketTypes::String | BucketTypes::Undefined => None,
        }
    }

    /// Cosine
    pub fn cos(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                ConstantTypes::E | ConstantTypes::TAU | ConstantTypes::C | ConstantTypes::G | ConstantTypes::PHI => {
                    Some(Self::from(
                        self.value.clone()?.parse::<f64>().unwrap().cos(),
                    ))
                }
                ConstantTypes::PI => Some(Self::from(-1)),
                ConstantTypes::TwoPI => Some(Self::from(1)),
                ConstantTypes::HalfPI => Some(Self::from(0)),
                ConstantTypes::QuarterPI => Some(Self::from(consts::FRAC_1_SQRT_2)),
                ConstantTypes::EighthPI => Some(Self::from(consts::FRAC_PI_8.cos())),
                ConstantTypes::SixthPI => Some(Self::from(consts::FRAC_PI_6.cos())),
                ConstantTypes::ThirdPI => Some(Self::from(0.5)),
            },
            BucketTypes::Float => Some(Self::from(
                Decimal::from_f64(self.value.clone()?.parse::<f64>().unwrap())?.checked_cos()?,
            )),
            BucketTypes::String | BucketTypes::Undefined => None,
        }
    }

    /// Tangent
    pub fn tan(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                ConstantTypes::E | ConstantTypes::TAU | ConstantTypes::C | ConstantTypes::G | ConstantTypes::PHI => {
                    Some(Self::from(
                        self.value.clone()?.parse::<f64>().unwrap().tan(),
                    ))
                }
                ConstantTypes::PI => Some(Self::from(0)),
                ConstantTypes::TwoPI => Some(Self::from(0)),
                ConstantTypes::HalfPI => Some(Self::new_undefined()),
                ConstantTypes::QuarterPI => Some(Self::from(1)),
                ConstantTypes::EighthPI => Some(Self::from(consts::FRAC_PI_8.tan())),
                ConstantTypes::SixthPI => Some(Self::from(consts::FRAC_PI_6.tan())),
                ConstantTypes::ThirdPI => Some(Self::from(consts::FRAC_PI_3.tan())),
            },
            BucketTypes::Float => Some(Self::from(
                Decimal::from_f64(self.value.clone()?.parse::<f64>().unwrap())?.checked_tan()?,
            )),
            BucketTypes::String | BucketTypes::Undefined => None,
        }
    }

    /// Cosecant
    pub fn csc(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                // Compute:
                // 1 / sin(value)
                ConstantTypes::E | ConstantTypes::TAU | ConstantTypes::C | ConstantTypes::G | ConstantTypes::PHI => {
                    Some(Self::from(
                        dec!(1.0)
                            / Decimal::from_f64(self.value.clone()?.parse::<f64>().unwrap())?
                                .checked_sin()?,
                    ))
                }
                ConstantTypes::PI | ConstantTypes::TwoPI => Some(Self::new_undefined()),
                ConstantTypes::HalfPI => Some(Self::from(1)),
                ConstantTypes::QuarterPI => Some(Self::from(consts::SQRT_2)),
                ConstantTypes::EighthPI => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_8.sin())?,
                )),
                ConstantTypes::SixthPI => Some(Self::from(2)),
                ConstantTypes::ThirdPI => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_3.sin())?,
                )),
            },
            BucketTypes::Float => match &self.value {
                Some(value) => {
                    let float_value = value.parse::<f64>().unwrap();
                    if float_value == 0.0 {
                        Some(Self::new_undefined())
                    } else {
                        Some(Self::from(
                            dec!(1.0) / Decimal::from_f64(float_value)?.checked_sin()?,
                        ))
                    }
                }
                None => None,
            },
            BucketTypes::String | BucketTypes::Undefined => None,
        }
    }

    /// Secant
    pub fn sec(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                // Compute:
                // 1 / cos(value)
                ConstantTypes::E | ConstantTypes::TAU | ConstantTypes::C | ConstantTypes::G | ConstantTypes::PHI => {
                    Some(Self::from(
                        dec!(1.0)
                            / Decimal::from_f64(self.value.clone()?.parse::<f64>().unwrap())?
                                .checked_cos()?,
                    ))
                }
                ConstantTypes::PI => Some(Self::from(-1)),
                ConstantTypes::TwoPI => Some(Self::from(1)),
                ConstantTypes::HalfPI => Some(Self::new_undefined()),
                ConstantTypes::QuarterPI => Some(Self::from(consts::SQRT_2)),
                ConstantTypes::EighthPI => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_8.cos())?,
                )),
                ConstantTypes::SixthPI => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_6.cos())?,
                )),
                ConstantTypes::ThirdPI => Some(Self::from(2)),
            },
            BucketTypes::Float => match &self.value {
                Some(value) => {
                    let float_value = value.parse::<f64>().unwrap();
                    // check if equal to 3pi/2
                    if float_value == (3.0 * consts::PI) / 2.0 {
                        Some(Self::new_undefined())
                    } else {
                        Some(Self::from(
                            dec!(1.0) / Decimal::from_f64(float_value)?.checked_sin()?,
                        ))
                    }
                }
                None => None,
            },
            BucketTypes::String | BucketTypes::Undefined => None,
        }
    }

    /// Cotangent
    pub fn cot(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                // Compute:
                // 1 / tan(value)
                ConstantTypes::E | ConstantTypes::TAU | ConstantTypes::C | ConstantTypes::G | ConstantTypes::PHI => {
                    Some(Self::from(
                        dec!(1.0)
                            / Decimal::from_f64(self.value.clone()?.parse::<f64>().unwrap())?
                                .checked_tan()?,
                    ))
                }
                ConstantTypes::PI | ConstantTypes::TwoPI => Some(Self::new_undefined()),
                ConstantTypes::HalfPI => Some(Self::from(0)),
                ConstantTypes::QuarterPI => Some(Self::from(1)),
                ConstantTypes::EighthPI => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_8.tan())?,
                )),
                ConstantTypes::SixthPI => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_6.tan())?,
                )),
                ConstantTypes::ThirdPI => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_3.tan())?,
                )),
            },
            BucketTypes::Float => match &self.value {
                Some(value) => {
                    let float_value = value.parse::<f64>().unwrap();
                    if float_value == 0.0 {
                        Some(Self::new_undefined())
                    } else {
                        Some(Self::from(
                            dec!(1.0) / Decimal::from_f64(float_value)?.checked_sin()?,
                        ))
                    }
                }
                None => None,
            },
            BucketTypes::String | BucketTypes::Undefined => None,
        }
    }
}

// implementation of .to_string()
impl ToString for Bucket {
    fn to_string(&self) -> String {
        match &self.value {
            Some(value) => value.to_string(),
            None => "Undefined".to_string(),
        }
    }
}

// float and integer implementations of from
macro_rules! generate_float_impl {
    ( $($t:ty),* ) => {
        $( impl From<$t> for Bucket {
            fn from(value: $t) -> Self {
                Self {
                    value: Some(value.to_string()),
                    bucket_type: BucketTypes::Float,
                }
            }
        } ) *
    };
}

macro_rules! generate_int_impl {
    ( $($t:ty),* ) => {
        $( impl From<$t> for Bucket {
            fn from(value: $t) -> Self {
                Self {
                    value: Some((value as f64).to_string()),
                    bucket_type: BucketTypes::Float,
                }
            }
        } ) *
    };
}

generate_float_impl! {f32, f64}
generate_int_impl! { u8, u16, u32, u64, i8, i16, i32, i64 }

impl From<Decimal> for Bucket {
    fn from(value: Decimal) -> Self {
        Self {
            value: Some(value.to_string()),
            bucket_type: BucketTypes::Float,
        }
    }
}

// string implementation of from
impl From<String> for Bucket {
    fn from(value: String) -> Self {
        Self {
            value: Some(value),
            bucket_type: BucketTypes::String,
        }
    }
}

impl From<&str> for Bucket {
    fn from(value: &str) -> Self {
        Self {
            value: Some(value.to_owned()),
            bucket_type: BucketTypes::String,
        }
    }
}

// serialize and deserialize for serde
impl Serialize for Bucket {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Bucket {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let deserialized_string = deserializer.deserialize_string(BucketVisitor)?;
        Ok(Self::from(deserialized_string))
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Default implementation just delegates to `deserialize` impl.
        *place = Deserialize::deserialize(deserializer)?;
        Ok(())
    }
}

struct BucketVisitor;
impl<'de> Visitor<'de> for BucketVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a String")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }
}
