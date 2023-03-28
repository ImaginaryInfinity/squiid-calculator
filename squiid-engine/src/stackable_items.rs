use crate::stackable_items::StackableItems::{StackableFloat, StackableString};

// extract the value within a StackableItems type
macro_rules! into_raw(
    ($value:expr, $expected_type:tt) => {
        match $value {
            $expected_type(i) => i,
            value => panic!("Tried to use a {:?} as a {}", value, stringify!($expected_type)),
        }
    }
);
pub(crate) use into_raw;

// items that can be added to the stack safely
#[derive(Debug, Clone, PartialEq)]
pub enum StackableItems {
    StackableFloat(f64),
    StackableString(String),
}

// implementation of arithmetic functions
impl StackableItems {
    pub fn powf(&self, n: StackableItems) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i.powf(into_raw!(n, StackableFloat))),
            StackableString(_) => panic!("cannot powf a string"),
        }
    }

    pub fn sqrt(&self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i.sqrt()),
            StackableString(_) => panic!("cannot powf a string"),
        }
    }

    pub fn add(&self, other: Self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i + into_raw!(other, StackableFloat)),
            StackableString(i) => {
                StackableString(i.to_owned() + &into_raw!(other, StackableString))
            }
        }
    }

    pub fn sub(&self, other: Self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i - into_raw!(other, StackableFloat)),
            StackableString(_) => panic!("Cannot subtract two strings"),
        }
    }

    pub fn div(&self, other: Self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i / into_raw!(other, StackableFloat)),
            StackableString(_) => panic!("Cannot divide two strings"),
        }
    }

    pub fn mul(&self, other: Self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i * into_raw!(other, StackableFloat)),
            StackableString(_) => panic!("Cannot multiply two strings"),
        }
    }

    pub fn modulo(&self, other: Self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i % into_raw!(other, StackableFloat)),
            StackableString(_) => panic!("Cannot modulo two strings"),
        }
    }

    pub fn sin(&self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i.sin()),
            StackableString(_) => panic!("cannot perform trigonometry on a string"),
        }
    }

    pub fn cos(&self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i.cos()),
            StackableString(_) => panic!("cannot perform trigonometry on a string"),
        }
    }

    pub fn tan(&self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i.tan()),
            StackableString(_) => panic!("cannot perform trigonometry on a string"),
        }
    }

    pub fn sec(&self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(1.0 / i.cos()),
            StackableString(_) => panic!("cannot perform trigonometry on a string"),
        }
    }

    pub fn csc(&self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(1.0 / i.sin()),
            StackableString(_) => panic!("cannot perform trigonometry on a string"),
        }
    }

    pub fn cot(&self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(1.0 / i.tan()),
            StackableString(_) => panic!("cannot perform trigonometry on a string"),
        }
    }

    pub fn asin(&self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i.asin()),
            StackableString(_) => panic!("cannot perform trigonometry on a string"),
        }
    }

    pub fn acos(&self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i.acos()),
            StackableString(_) => panic!("cannot perform trigonometry on a string"),
        }
    }

    pub fn atan(&self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i.atan()),
            StackableString(_) => panic!("cannot perform trigonometry on a string"),
        }
    }

    pub fn log(&self, other: Self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i.log(into_raw!(other, StackableFloat))),
            StackableString(_) => panic!("cannot perform logarithm on a string"),
        }
    }

    pub fn ln(&self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i.ln()),
            StackableString(_) => panic!("cannot perform logarithm on a string"),
        }
    }

    pub fn abs(&self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i.abs()),
            StackableString(_) => panic!("cannot perform absolute value on a string"),
        }
    }

    pub fn eq(&self, other: Self) -> Self {
        match self {
            StackableFloat(i) => {
                StackableFloat((i == &(into_raw!(other, StackableFloat))) as i32 as f64)
            }
            StackableString(_) => panic!("cannot compare strings"),
        }
    }

    pub fn gt(&self, other: Self) -> Self {
        match self {
            StackableFloat(i) => {
                StackableFloat((i > &(into_raw!(other, StackableFloat))) as i32 as f64)
            }
            StackableString(_) => panic!("cannot compare strings"),
        }
    }

    pub fn lt(&self, other: Self) -> Self {
        match self {
            StackableFloat(i) => {
                StackableFloat((i < &(into_raw!(other, StackableFloat))) as i32 as f64)
            }
            StackableString(_) => panic!("cannot compare strings"),
        }
    }

    pub fn gte(&self, other: Self) -> Self {
        match self {
            StackableFloat(i) => {
                StackableFloat((i >= &(into_raw!(other, StackableFloat))) as i32 as f64)
            }
            StackableString(_) => panic!("cannot compare strings"),
        }
    }

    pub fn lte(&self, other: Self) -> Self {
        match self {
            StackableFloat(i) => {
                StackableFloat((i <= &(into_raw!(other, StackableFloat))) as i32 as f64)
            }
            StackableString(_) => panic!("cannot compare strings"),
        }
    }

    pub fn round(&self) -> Self {
        match self {
            StackableFloat(i) => StackableFloat(i.round()),
            StackableString(_) => panic!("cannot round strings"),
        }
    }
}

// implementation of .to_string()
impl ToString for StackableItems {
    fn to_string(&self) -> String {
        match self {
            StackableFloat(i) => i.to_string(),
            StackableString(i) => i.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::*;

    #[test]
    fn create_stackable_item() {
        let _ = StackableFloat(3.0);
        let _ = StackableString(String::from("test"));
    }

    #[test]
    fn test_into_raw() {
        let stackable_float = StackableFloat(3.0);
        assert_eq!(into_raw!(stackable_float, StackableFloat), 3.0);

        let stackable_string = StackableString(String::from("test"));
        assert_eq!(into_raw!(stackable_string, StackableString), "test");
    }

    #[test]
    fn test_to_string() {
        let stackable_float = StackableFloat(3.0);
        let stackable_string = StackableString(String::from("test"));

        assert_eq!(stackable_float.to_string(), "3");
        assert_eq!(stackable_string.to_string(), "test");
    }

    #[test]
    fn test_algebra() {
        // powf
        assert_eq!(
            StackableFloat(3.0).powf(StackableFloat(3.0)),
            StackableFloat(27.0)
        );

        // sqrt
        assert_eq!(StackableFloat(9.0).sqrt(), StackableFloat(3.0));

        // add
        assert_eq!(
            StackableFloat(3.0).add(StackableFloat(3.0)),
            StackableFloat(6.0)
        );

        // sub
        assert_eq!(
            StackableFloat(3.0).sub(StackableFloat(3.0)),
            StackableFloat(0.0)
        );

        // div
        assert_eq!(
            StackableFloat(3.0).div(StackableFloat(3.0)),
            StackableFloat(1.0)
        );

        // mul
        assert_eq!(
            StackableFloat(3.0).mul(StackableFloat(3.0)),
            StackableFloat(9.0)
        );

        // modulo
        assert_eq!(
            StackableFloat(4.0).modulo(StackableFloat(3.0)),
            StackableFloat(1.0)
        );

        // rounding
        assert_eq!(StackableFloat(4.3).round(), StackableFloat(4.0));
        assert_eq!(StackableFloat(4.6).round(), StackableFloat(5.0));

        // sin
        assert_eq!(StackableFloat(PI).sin().round(), StackableFloat(0.0));

        // cos
        assert_eq!(StackableFloat(PI).cos(), StackableFloat(-1.0));

        // tan
        assert_eq!(StackableFloat(PI).tan().round(), StackableFloat(0.0));

        // sec
        assert_eq!(StackableFloat(PI).sec(), StackableFloat(-1.0));

        // csc
        assert_eq!(StackableFloat(PI / 2.0).csc(), StackableFloat(1.0));

        // cot
        assert_eq!(StackableFloat(PI / 2.0).cot().round(), StackableFloat(0.0));

        // asin
        assert_eq!(StackableFloat(0.0).asin(), StackableFloat(0.0));

        // acos
        assert_eq!(StackableFloat(0.0).acos(), StackableFloat(PI / 2.0));

        // atan
        assert_eq!(StackableFloat(0.0).atan(), StackableFloat(0.0));

        // log
        assert_eq!(
            StackableFloat(1.0).log(StackableFloat(10.0)),
            StackableFloat(0.0)
        );

        // ln
        assert_eq!(StackableFloat(1.0).ln(), StackableFloat(0.0));

        // abs
        assert_eq!(StackableFloat(-1.0).abs(), StackableFloat(1.0));
        assert_eq!(StackableFloat(1.0).abs(), StackableFloat(1.0));

        // eq
        assert_eq!(
            StackableFloat(13.2).eq(StackableFloat(13.2)),
            StackableFloat(1.0)
        );
        assert_eq!(
            StackableFloat(13.2).eq(StackableFloat(1.0)),
            StackableFloat(0.0)
        );

        // gt
        assert_eq!(
            StackableFloat(1.0).gt(StackableFloat(0.0)),
            StackableFloat(1.0)
        );
        assert_eq!(
            StackableFloat(1.0).gt(StackableFloat(2.0)),
            StackableFloat(0.0)
        );

        // lt
        assert_eq!(
            StackableFloat(0.0).lt(StackableFloat(1.0)),
            StackableFloat(1.0)
        );
        assert_eq!(
            StackableFloat(2.0).lt(StackableFloat(1.0)),
            StackableFloat(0.0)
        );

        // gte
        assert_eq!(
            StackableFloat(1.0).gte(StackableFloat(0.0)),
            StackableFloat(1.0)
        );
        assert_eq!(
            StackableFloat(1.0).gte(StackableFloat(1.0)),
            StackableFloat(1.0)
        );
        assert_eq!(
            StackableFloat(1.0).gte(StackableFloat(2.0)),
            StackableFloat(0.0)
        );

        // lte
        assert_eq!(
            StackableFloat(0.0).lte(StackableFloat(1.0)),
            StackableFloat(1.0)
        );
        assert_eq!(
            StackableFloat(1.0).lte(StackableFloat(1.0)),
            StackableFloat(1.0)
        );
        assert_eq!(
            StackableFloat(2.0).lte(StackableFloat(1.0)),
            StackableFloat(0.0)
        );
    }
}
