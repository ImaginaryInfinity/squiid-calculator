// items on the stack are called Buckets

use std::f64::consts;

use rust_decimal::{prelude::FromPrimitive, Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum BucketTypes {
    Float,
    String,
    Constant(ConstantTypes),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bucket {
    pub value: String,
    pub bucket_type: BucketTypes,
}

impl Bucket {
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
        }
        .to_string();

        Bucket {
            value: value,
            bucket_type: BucketTypes::Constant(constant_type),
        }
    }

    pub fn sin(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                ConstantTypes::E | ConstantTypes::TAU | ConstantTypes::C | ConstantTypes::G => {
                    Some(Self::from(self.value.parse::<f64>().unwrap().sin()))
                }
                ConstantTypes::PI => Some(Self::from(Decimal::PI.checked_sin()?)),
                ConstantTypes::TwoPI => Some(Self::from(Decimal::TWO_PI.checked_sin()?)),
                ConstantTypes::HalfPI => Some(Self::from(Decimal::HALF_PI.checked_sin()?)),
                ConstantTypes::QuarterPI => Some(Self::from(Decimal::QUARTER_PI.checked_sin()?)),
                ConstantTypes::EighthPI => Some(Self::from(consts::FRAC_PI_8.sin())),
                ConstantTypes::SixthPI => Some(Self::from(consts::FRAC_PI_6.sin())),
                ConstantTypes::ThirdPI => Some(Self::from(consts::FRAC_PI_3.sin())),
            },
            BucketTypes::Float => Some(Self::from(
                Decimal::from_f64(self.value.parse::<f64>().unwrap())?.checked_sin()?,
            )),
            BucketTypes::String => None,
        }
    }

    pub fn cos(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                ConstantTypes::E | ConstantTypes::TAU | ConstantTypes::C | ConstantTypes::G => {
                    Some(Self::from(self.value.parse::<f64>().unwrap().cos()))
                }
                ConstantTypes::PI => Some(Self::from(Decimal::PI.checked_cos()?)),
                ConstantTypes::TwoPI => Some(Self::from(Decimal::TWO_PI.checked_cos()?)),
                ConstantTypes::HalfPI => Some(Self::from(Decimal::HALF_PI.checked_cos()?)),
                ConstantTypes::QuarterPI => Some(Self::from(Decimal::QUARTER_PI.checked_cos()?)),
                ConstantTypes::EighthPI => Some(Self::from(consts::FRAC_PI_8.cos())),
                ConstantTypes::SixthPI => Some(Self::from(consts::FRAC_PI_6.cos())),
                ConstantTypes::ThirdPI => Some(Self::from(consts::FRAC_PI_3.cos())),
            },
            BucketTypes::Float => Some(Self::from(
                Decimal::from_f64(self.value.parse::<f64>().unwrap())?.checked_cos()?,
            )),
            BucketTypes::String => None,
        }
    }

    pub fn tan(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                ConstantTypes::E | ConstantTypes::TAU | ConstantTypes::C | ConstantTypes::G => {
                    Some(Self::from(self.value.parse::<f64>().unwrap().tan()))
                }
                ConstantTypes::PI => Some(Self::from(Decimal::PI.checked_tan()?)),
                ConstantTypes::TwoPI => Some(Self::from(Decimal::TWO_PI.checked_tan()?)),
                ConstantTypes::HalfPI => Some(Self::from(Decimal::HALF_PI.checked_tan()?)),
                ConstantTypes::QuarterPI => Some(Self::from(Decimal::QUARTER_PI.checked_tan()?)),
                ConstantTypes::EighthPI => Some(Self::from(consts::FRAC_PI_8.tan())),
                ConstantTypes::SixthPI => Some(Self::from(consts::FRAC_PI_6.tan())),
                ConstantTypes::ThirdPI => Some(Self::from(consts::FRAC_PI_3.tan())),
            },
            BucketTypes::Float => Some(Self::from(
                Decimal::from_f64(self.value.parse::<f64>().unwrap())?.checked_tan()?,
            )),
            BucketTypes::String => None,
        }
    }

    pub fn csc(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                // Compute:
                // 1 / sin(value)
                ConstantTypes::E | ConstantTypes::TAU | ConstantTypes::C | ConstantTypes::G => {
                    Some(Self::from(
                        dec!(1.0)
                            / Decimal::from_f64(self.value.parse::<f64>().unwrap())?
                                .checked_sin()?,
                    ))
                }
                ConstantTypes::PI => Some(Self::from(dec!(1.0) / Decimal::PI.checked_sin()?)),
                ConstantTypes::TwoPI => {
                    Some(Self::from(dec!(1.0) / Decimal::TWO_PI.checked_sin()?))
                }
                ConstantTypes::HalfPI => {
                    Some(Self::from(dec!(1.0) / Decimal::HALF_PI.checked_sin()?))
                }
                ConstantTypes::QuarterPI => {
                    Some(Self::from(dec!(1.0) / Decimal::QUARTER_PI.checked_sin()?))
                }
                ConstantTypes::EighthPI => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_8.sin())?,
                )),
                ConstantTypes::SixthPI => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_6.sin())?,
                )),
                ConstantTypes::ThirdPI => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_3.sin())?,
                )),
            },
            BucketTypes::Float => Some(Self::from(
                dec!(1.0) / Decimal::from_f64(self.value.parse::<f64>().unwrap())?.checked_sin()?,
            )),
            BucketTypes::String => None,
        }
    }

    pub fn sec(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                // Compute:
                // 1 / cos(value)
                ConstantTypes::E | ConstantTypes::TAU | ConstantTypes::C | ConstantTypes::G => {
                    Some(Self::from(
                        dec!(1.0)
                            / Decimal::from_f64(self.value.parse::<f64>().unwrap())?
                                .checked_cos()?,
                    ))
                }
                ConstantTypes::PI => Some(Self::from(dec!(1.0) / Decimal::PI.checked_cos()?)),
                ConstantTypes::TwoPI => {
                    Some(Self::from(dec!(1.0) / Decimal::TWO_PI.checked_cos()?))
                }
                ConstantTypes::HalfPI => {
                    Some(Self::from(dec!(1.0) / Decimal::HALF_PI.checked_cos()?))
                }
                ConstantTypes::QuarterPI => {
                    Some(Self::from(dec!(1.0) / Decimal::QUARTER_PI.checked_cos()?))
                }
                ConstantTypes::EighthPI => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_8.cos())?,
                )),
                ConstantTypes::SixthPI => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_6.cos())?,
                )),
                ConstantTypes::ThirdPI => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_3.cos())?,
                )),
            },
            BucketTypes::Float => Some(Self::from(
                dec!(1.0) / Decimal::from_f64(self.value.parse::<f64>().unwrap())?.checked_cos()?,
            )),
            BucketTypes::String => None,
        }
    }

    pub fn cot(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                // Compute:
                // 1 / tan(value)
                ConstantTypes::E | ConstantTypes::TAU | ConstantTypes::C | ConstantTypes::G => {
                    Some(Self::from(
                        dec!(1.0)
                            / Decimal::from_f64(self.value.parse::<f64>().unwrap())?
                                .checked_tan()?,
                    ))
                }
                ConstantTypes::PI => Some(Self::from(dec!(1.0) / Decimal::PI.checked_tan()?)),
                ConstantTypes::TwoPI => {
                    Some(Self::from(dec!(1.0) / Decimal::TWO_PI.checked_tan()?))
                }
                ConstantTypes::HalfPI => {
                    Some(Self::from(dec!(1.0) / Decimal::HALF_PI.checked_tan()?))
                }
                ConstantTypes::QuarterPI => {
                    Some(Self::from(dec!(1.0) / Decimal::QUARTER_PI.checked_tan()?))
                }
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
            BucketTypes::Float => Some(Self::from(
                dec!(1.0) / Decimal::from_f64(self.value.parse::<f64>().unwrap())?.checked_tan()?,
            )),
            BucketTypes::String => None,
        }
    }
}

// implementation of .to_string()
impl ToString for Bucket {
    fn to_string(&self) -> String {
        self.value.clone()
    }
}

// float and integer implementations of from
macro_rules! generate_float_impl {
    ( $($t:ty),* ) => {
        $( impl From<$t> for Bucket {
            fn from(value: $t) -> Self {
                Self {
                    value: value.to_string(),
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
                    value: (value as f64).to_string(),
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
            value: value.to_string(),
            bucket_type: BucketTypes::Float,
        }
    }
}

// string implementation of from
impl From<String> for Bucket {
    fn from(value: String) -> Self {
        Self {
            value: value,
            bucket_type: BucketTypes::String,
        }
    }
}

impl From<&str> for Bucket {
    fn from(value: &str) -> Self {
        Self {
            value: value.to_owned(),
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
