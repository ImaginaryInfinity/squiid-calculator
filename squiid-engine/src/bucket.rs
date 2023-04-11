// items on the stack are called Buckets

use rust_decimal::Decimal;
use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum BucketTypes {
    Float,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bucket {
    pub value: String,
    pub bucket_type: BucketTypes,
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
