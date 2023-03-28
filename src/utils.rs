// function to check if a string is numeric (includes floats)
pub fn is_string_numeric(str: &str) -> bool {
    for c in str.chars() {
        // If a character is not a number or contains only a decimal point, negative sign, or e, the string is not numeric
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_function() {
        assert_eq!(is_string_numeric("abc"), false);
        assert_eq!(is_string_numeric("1a"), false);
        assert_eq!(is_string_numeric("12e.a"), false);
        assert_eq!(is_string_numeric("123"), true);
        assert_eq!(is_string_numeric("1.2"), true);
        assert_eq!(is_string_numeric("1.2e7"), true);
    }
}
