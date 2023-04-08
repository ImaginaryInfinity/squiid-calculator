// items on the stack are called Buckets

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