// Left/Right Associativity enum for distinguising between right associative operations such as power
#[derive(PartialEq)]
pub enum Associativity {
    Left,
    Right,
}

// operator properites class that contains the operator precendence and the associativity
pub struct OperatorProperties {
    pub precedence: u8,
    pub associativity: Associativity,
}

// function to check if a string is numeric. _ indicates negative numbers
pub fn is_string_numeric(str: &str) -> bool {
    for c in str.chars() {
        // return false if the current character is not numeric or is not a valid numerical character, such as . for decimals and _ for indicative negatives
        if !c.is_numeric()
            && !(['.', '-', 'e'].contains(&c)
                && str.chars().count() > 1
                && !['-', 'e'].contains(&(str.chars().last().unwrap())))
        {
            return false;
        }
    }
    return true;
}

// function to check if a string is alphabetic
pub fn is_string_alphabetic(str: &str) -> bool {
    for c in str.chars() {
        if !c.is_alphabetic() {
            return false;
        }
    }
    return true;
}
