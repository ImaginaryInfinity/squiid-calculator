// items on the stack are called Buckets

use std::{collections::HashMap, f64::consts, fmt::Display, sync::LazyLock};

use rust_decimal::{prelude::FromPrimitive, Decimal, MathematicalOps};
use rust_decimal_macros::dec;

/// Types of constants
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ConstantTypes {
    /// Pi
    Pi,
    /// Pi/2
    HalfPi,
    /// Pi/3
    ThirdPi,
    /// Pi/4
    QuarterPi,
    /// Pi/6
    SixthPi,
    /// Pi/8
    EighthPi,
    /// 2*pi
    TwoPi,
    /// Euler's number
    E,
    /// Speed of light
    C,
    /// Gravitational constant
    G,
    /// Golden ratio
    Phi,
}

pub static CONSTANT_IDENTIFIERS: LazyLock<HashMap<&'static str, ConstantTypes>> =
    LazyLock::new(|| {
        HashMap::from([
            ("#pi", ConstantTypes::Pi),
            ("#e", ConstantTypes::E),
            ("#tau", ConstantTypes::TwoPi),
            ("#c", ConstantTypes::C),
            ("#G", ConstantTypes::G),
            ("#phi", ConstantTypes::Phi),
            ("#halfpi", ConstantTypes::HalfPi),
            ("#thirdpi", ConstantTypes::ThirdPi),
            ("#quarterpi", ConstantTypes::QuarterPi),
            ("#sixthpi", ConstantTypes::SixthPi),
            ("#eighthpi", ConstantTypes::EighthPi),
            ("#twopi", ConstantTypes::TwoPi),
        ])
    });

/// Types of Buckets
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum BucketTypes {
    /// A floating point number. Also contains integers such as 3.0
    Float,
    /// A string
    String,
    /// A constant
    Constant(ConstantTypes),
    /// Undefined value, such as tan(pi/2)
    // TODO: should undefined error out? in trig and stuff
    Undefined,
}

/// Bucket contains the items that can be on the stack
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
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
            ConstantTypes::Pi => consts::PI,
            ConstantTypes::HalfPi => consts::FRAC_PI_2,
            ConstantTypes::ThirdPi => consts::FRAC_PI_3,
            ConstantTypes::QuarterPi => consts::FRAC_PI_4,
            ConstantTypes::SixthPi => consts::FRAC_PI_6,
            ConstantTypes::EighthPi => consts::FRAC_PI_8,
            ConstantTypes::TwoPi => consts::TAU,
            ConstantTypes::E => consts::E,
            ConstantTypes::C => 299_792_458_f64,
            ConstantTypes::G => 6.67430 * 10_f64.powf(-11_f64),
            ConstantTypes::Phi => 1.618_033_988_749_895_f64,
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
                ConstantTypes::E | ConstantTypes::C | ConstantTypes::G | ConstantTypes::Phi => {
                    Some(Self::from(self.value.clone()?.parse::<f64>().ok()?.sin()))
                }
                ConstantTypes::Pi | ConstantTypes::TwoPi => Some(Self::from(0)),
                ConstantTypes::HalfPi => Some(Self::from(1)),
                ConstantTypes::QuarterPi => Some(Self::from(consts::FRAC_1_SQRT_2)),
                ConstantTypes::EighthPi => Some(Self::from(consts::FRAC_PI_8.sin())),
                ConstantTypes::SixthPi => Some(Self::from(0.5)),
                ConstantTypes::ThirdPi => Some(Self::from(consts::FRAC_PI_3.sin())),
            },
            BucketTypes::Float => Some(Self::from(
                Decimal::from_f64(self.value.clone()?.parse::<f64>().ok()?)?.checked_sin()?,
            )),
            BucketTypes::String | BucketTypes::Undefined => None,
        }
    }

    /// Cosine
    pub fn cos(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                ConstantTypes::E | ConstantTypes::C | ConstantTypes::G | ConstantTypes::Phi => {
                    Some(Self::from(self.value.clone()?.parse::<f64>().ok()?.cos()))
                }
                ConstantTypes::Pi => Some(Self::from(-1)),
                ConstantTypes::TwoPi => Some(Self::from(1)),
                ConstantTypes::HalfPi => Some(Self::from(0)),
                ConstantTypes::QuarterPi => Some(Self::from(consts::FRAC_1_SQRT_2)),
                ConstantTypes::EighthPi => Some(Self::from(consts::FRAC_PI_8.cos())),
                ConstantTypes::SixthPi => Some(Self::from(consts::FRAC_PI_6.cos())),
                ConstantTypes::ThirdPi => Some(Self::from(0.5)),
            },
            BucketTypes::Float => Some(Self::from(
                Decimal::from_f64(self.value.clone()?.parse::<f64>().ok()?)?.checked_cos()?,
            )),
            BucketTypes::String | BucketTypes::Undefined => None,
        }
    }

    /// Tangent
    pub fn tan(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                ConstantTypes::E | ConstantTypes::C | ConstantTypes::G | ConstantTypes::Phi => {
                    Some(Self::from(self.value.clone()?.parse::<f64>().ok()?.tan()))
                }
                ConstantTypes::Pi | ConstantTypes::TwoPi => Some(Self::from(0)),
                ConstantTypes::HalfPi => Some(Self::new_undefined()),
                ConstantTypes::QuarterPi => Some(Self::from(1)),
                ConstantTypes::EighthPi => Some(Self::from(consts::FRAC_PI_8.tan())),
                ConstantTypes::SixthPi => Some(Self::from(consts::FRAC_PI_6.tan())),
                ConstantTypes::ThirdPi => Some(Self::from(consts::FRAC_PI_3.tan())),
            },
            BucketTypes::Float => match &self.value {
                Some(value) => {
                    let float_value = value.parse::<f64>().ok()?;
                    // check if equal to 3pi/2
                    if float_value == (3.0 * consts::PI) / 2.0 {
                        Some(Self::new_undefined())
                    } else {
                        Some(Self::from(
                            Decimal::from_f64(self.value.clone()?.parse::<f64>().ok()?)?
                                .checked_tan()?,
                        ))
                    }
                }
                None => None,
            },
            BucketTypes::String | BucketTypes::Undefined => None,
        }
    }

    /// Cosecant
    pub fn csc(&self) -> Option<Self> {
        match &self.bucket_type {
            BucketTypes::Constant(constant_type) => match constant_type {
                // Compute:
                // 1 / sin(value)
                ConstantTypes::E | ConstantTypes::C | ConstantTypes::G | ConstantTypes::Phi => {
                    Some(Self::from(
                        dec!(1.0)
                            / Decimal::from_f64(self.value.clone()?.parse::<f64>().ok()?)?
                                .checked_sin()?,
                    ))
                }
                ConstantTypes::Pi | ConstantTypes::TwoPi => Some(Self::new_undefined()),
                ConstantTypes::HalfPi => Some(Self::from(1)),
                ConstantTypes::QuarterPi => Some(Self::from(consts::SQRT_2)),
                ConstantTypes::EighthPi => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_8.sin())?,
                )),
                ConstantTypes::SixthPi => Some(Self::from(2)),
                ConstantTypes::ThirdPi => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_3.sin())?,
                )),
            },
            BucketTypes::Float => match &self.value {
                Some(value) => {
                    let float_value = value.parse::<f64>().ok()?;
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
                ConstantTypes::E | ConstantTypes::C | ConstantTypes::G | ConstantTypes::Phi => {
                    Some(Self::from(
                        dec!(1.0)
                            / Decimal::from_f64(self.value.clone()?.parse::<f64>().ok()?)?
                                .checked_cos()?,
                    ))
                }
                ConstantTypes::Pi => Some(Self::from(-1)),
                ConstantTypes::TwoPi => Some(Self::from(1)),
                ConstantTypes::HalfPi => Some(Self::new_undefined()),
                ConstantTypes::QuarterPi => Some(Self::from(consts::SQRT_2)),
                ConstantTypes::EighthPi => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_8.cos())?,
                )),
                ConstantTypes::SixthPi => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_6.cos())?,
                )),
                ConstantTypes::ThirdPi => Some(Self::from(2)),
            },
            BucketTypes::Float => match &self.value {
                Some(value) => {
                    let float_value = value.parse::<f64>().ok()?;

                    // Handle sec(0) correctly, which should return 1
                    if float_value == 0.0 {
                        Some(Self::from(1)) // sec(0) = 1
                    } else if float_value == (3.0 * consts::PI) / 2.0 {
                        Some(Self::new_undefined()) // sec(3#pi/2) = undefined
                    } else {
                        Some(Self::from(
                            dec!(1.0) / Decimal::from_f64(float_value)?.checked_cos()?,
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
                ConstantTypes::E | ConstantTypes::C | ConstantTypes::G | ConstantTypes::Phi => {
                    Some(Self::from(
                        dec!(1.0)
                            / Decimal::from_f64(self.value.clone()?.parse::<f64>().ok()?)?
                                .checked_tan()?,
                    ))
                }
                ConstantTypes::Pi | ConstantTypes::TwoPi => Some(Self::new_undefined()),
                ConstantTypes::HalfPi => Some(Self::from(0)),
                ConstantTypes::QuarterPi => Some(Self::from(1)),
                ConstantTypes::EighthPi => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_8.tan())?,
                )),
                ConstantTypes::SixthPi => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_6.tan())?,
                )),
                ConstantTypes::ThirdPi => Some(Self::from(
                    dec!(1.0) / Decimal::from_f64(consts::FRAC_PI_3.tan())?,
                )),
            },
            BucketTypes::Float => match &self.value {
                Some(value) => {
                    let float_value = value.parse::<f64>().ok()?;
                    if float_value == 0.0 {
                        Some(Self::new_undefined())
                    } else {
                        Some(Self::from(
                            dec!(1.0) / Decimal::from_f64(float_value)?.checked_tan()?,
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
impl Display for Bucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match &self.value {
            Some(value) => value.to_string(),
            None => "Undefined".to_owned(),
        })
    }
}

/// Generate `From<>` implementations of floating points for [`Bucket`]
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

/// Generate `From<>` implementations of integers for [`Bucket`]
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
