use std::f64::consts::PI;

use squiid_engine::{
    bucket::{Bucket, BucketTypes, ConstantTypes},
    engine::*,
};

#[test]
fn test_create_engine() {
    Engine::new();
}

#[test]
fn test_add_negative_to_stack() {
    let mut engine = Engine::new();

    // negatives
    let _ = engine.add_item_to_stack("-1".into(), false);
    let _ = engine.add_item_to_stack("1".into(), false);

    assert_eq!(engine.stack, vec![Bucket::from(-1), Bucket::from(1)]);
}

#[test]
fn test_add_constants_to_stack() {
    let mut engine = Engine::new();

    // constants
    let _ = engine.add_item_to_stack("#pi".into(), false);
    let _ = engine.add_item_to_stack("#e".into(), false);
    let _ = engine.add_item_to_stack("#tau".into(), false);
    let _ = engine.add_item_to_stack("#c".into(), false);
    let _ = engine.add_item_to_stack("#G".into(), false);

    assert_eq!(
        engine.stack,
        vec![
            Bucket::from_constant(ConstantTypes::PI),
            Bucket::from_constant(ConstantTypes::E),
            Bucket::from_constant(ConstantTypes::TAU),
            Bucket::from_constant(ConstantTypes::C),
            Bucket::from_constant(ConstantTypes::G),
        ]
    );
}

#[test]
fn test_add_undefined_variable_to_stack() {
    let mut engine = Engine::new();

    let result = engine.add_item_to_stack("$a".into(), false);

    assert!(matches!(result, Err(_)));
}

#[test]
fn test_add_defined_variable_to_stack() {
    let mut engine = Engine::new();

    engine.variables.insert(String::from("a"), 1.into());
    let _ = engine.add_item_to_stack("$a".into(), false);

    assert_eq!(engine.stack, vec![Bucket::from(1)]);
}

#[test]
fn test_add_types_to_stack() {
    // test for adding string vs number to stack
    // we dont need extensive testing since is_string_numeric already
    // has tests written for it

    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("a".into(), false);
    let _ = engine.add_item_to_stack("1.5".into(), false);

    assert_eq!(engine.stack, vec![Bucket::from("a"), Bucket::from(1.5),]);
}

#[test]
fn test_get_operands() {
    let mut engine = Engine::new();

    // to get as floats
    let _ = engine.add_item_to_stack("1".into(), false);
    let _ = engine.add_item_to_stack("1.5".into(), false);
    let _ = engine.add_item_to_stack("a".into(), false);

    // to get as strings
    let _ = engine.add_item_to_stack("abc".into(), false);
    let _ = engine.add_item_to_stack("1.5".into(), false);

    // to get raw
    let _ = engine.add_item_to_stack("test".into(), false);
    let _ = engine.add_item_to_stack("1.5".into(), false);

    // retrieve things off of stack
    let raw = engine.get_operands_raw(2);
    let strings = engine.get_operands_as_string(2);
    let invalid_float = engine.get_operands_as_f(1);
    let _ = engine.get_operands_as_string(1); // clear the invalid float off the stack
    let valid_floats = engine.get_operands_as_f(2);

    assert_eq!(raw, Ok(vec![Bucket::from("test"), Bucket::from(1.5),]));

    assert_eq!(strings, Ok(vec![String::from("abc"), String::from("1.5"),]));

    assert!(matches!(invalid_float, Err(_)));

    assert_eq!(valid_floats, Ok(vec![1.0, 1.5,]));
}

#[test]
fn test_add() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("1".into(), false);
    let _ = engine.add_item_to_stack("1".into(), false);

    let _ = engine.add_item_to_stack("-1".into(), false);
    let _ = engine.add_item_to_stack("1".into(), false);

    let _ = engine.add_item_to_stack("-1".into(), false);
    let _ = engine.add_item_to_stack("-1".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.add();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], -2.0);

    let _ = engine.add();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);

    let _ = engine.add();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 2.0);
}

#[test]
fn test_subtract() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("1".into(), false);
    let _ = engine.add_item_to_stack("1".into(), false);

    let _ = engine.add_item_to_stack("-1".into(), false);
    let _ = engine.add_item_to_stack("1".into(), false);

    let _ = engine.add_item_to_stack("-1".into(), false);
    let _ = engine.add_item_to_stack("-1".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.subtract();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);

    let _ = engine.subtract();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], -2.0);

    let _ = engine.subtract();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);
}

#[test]
fn test_multiply() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("2".into(), false);
    let _ = engine.add_item_to_stack("3".into(), false);

    let _ = engine.add_item_to_stack("-1".into(), false);
    let _ = engine.add_item_to_stack("2".into(), false);

    let _ = engine.add_item_to_stack("-2".into(), false);
    let _ = engine.add_item_to_stack("-4".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.multiply();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 8.0);

    let _ = engine.multiply();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], -2.0);

    let _ = engine.multiply();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 6.0);
}

#[test]
fn test_divide() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("6".into(), false);
    let _ = engine.add_item_to_stack("2".into(), false);

    let _ = engine.add_item_to_stack("-1".into(), false);
    let _ = engine.add_item_to_stack("2".into(), false);

    let _ = engine.add_item_to_stack("-4".into(), false);
    let _ = engine.add_item_to_stack("-2".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.divide();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 2.0);

    let _ = engine.divide();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], -0.5);

    let _ = engine.divide();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 3.0);
}

#[test]
fn test_power() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("2".into(), false);
    let _ = engine.add_item_to_stack("3".into(), false);

    let _ = engine.add_item_to_stack("4".into(), false);
    let _ = engine.add_item_to_stack("0.5".into(), false);

    let _ = engine.add_item_to_stack("2".into(), false);
    let _ = engine.add_item_to_stack("-1".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.power();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.5);

    let _ = engine.power();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 2.0);

    let _ = engine.power();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 8.0);
}

#[test]
fn test_sqrt() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("4".into(), false);

    let _ = engine.add_item_to_stack("9".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.sqrt();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 3.0);

    let _ = engine.sqrt();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 2.0);
}

#[test]
fn test_modulo() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-9".into(), false);
    let _ = engine.add_item_to_stack("4".into(), false);

    let _ = engine.add_item_to_stack("8".into(), false);
    let _ = engine.add_item_to_stack("4".into(), false);

    let _ = engine.add_item_to_stack("9".into(), false);
    let _ = engine.add_item_to_stack("4".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.modulo();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0);

    let _ = engine.modulo();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);

    let _ = engine.modulo();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], -1.0);
}

#[test]
fn test_sine() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("#pi".into(), false);
    let _ = engine.add_item_to_stack("2".into(), false);

    let _ = engine.add_item_to_stack("2".into(), false);
    let _ = engine.add_item_to_stack("#pi".into(), false);

    // evaluate from last stack entries to first

    // 2pi = 0
    let _ = engine.multiply();
    let _ = engine.sin();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0].round(), 0.0);

    // pi/2 = 1
    let _ = engine.divide();
    let _ = engine.sin();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0);
}

#[test]
fn test_cosine() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("#pi".into(), false);
    let _ = engine.add_item_to_stack("2".into(), false);

    let _ = engine.add_item_to_stack("2".into(), false);
    let _ = engine.add_item_to_stack("#pi".into(), false);

    // evaluate from last stack entries to first

    // 2pi = 1
    let _ = engine.multiply();
    let _ = engine.cos();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0);

    // pi/2 = 0
    let _ = engine.divide();
    let _ = engine.cos();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0].round(), 0.0);
}

#[test]
fn test_tangent() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("#pi".into(), false);
    let _ = engine.add_item_to_stack("4".into(), false);

    let _ = engine.add_item_to_stack("2".into(), false);
    let _ = engine.add_item_to_stack("#pi".into(), false);

    // evaluate from last stack entries to first

    // 2pi = 0
    let _ = engine.multiply();
    let _ = engine.tan();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0].round(), 0.0);

    // pi/4 = 1
    let _ = engine.divide();
    let _ = engine.tan();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0].round(), 1.0);
}

#[test]
fn test_secant() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("#pi".into(), false);

    let _ = engine.add_item_to_stack("#pi".into(), false);
    let _ = engine.add_item_to_stack("3".into(), false);

    // evaluate from last stack entries to first

    // pi/3 = 2
    let _ = engine.divide();
    let _ = engine.sec();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0].round(), 2.0);

    // pi = -1
    let _ = engine.sec();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0].round(), -1.0);
}

#[test]
fn test_cosecant() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("#pi".into(), false);
    let _ = engine.add_item_to_stack("6".into(), false);

    let _ = engine.add_item_to_stack("#pi".into(), false);
    let _ = engine.add_item_to_stack("2".into(), false);

    // evaluate from last stack entries to first

    // pi/2 = 1
    let _ = engine.divide();
    let _ = engine.csc();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0].round(), 1.0);

    // pi/6 = 2
    let _ = engine.divide();
    let _ = engine.csc();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0].round(), 2.0);
}

#[test]
fn test_cotangent() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("#pi".into(), false);
    let _ = engine.add_item_to_stack("4".into(), false);

    let _ = engine.add_item_to_stack("#pi".into(), false);
    let _ = engine.add_item_to_stack("2".into(), false);

    // evaluate from last stack entries to first

    // pi/2 = 0
    let _ = engine.divide();
    let _ = engine.cot();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0].round(), 0.0);

    // pi/4 = 1
    let _ = engine.divide();
    let _ = engine.cot();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0].round(), 1.0);
}

#[test]
fn test_asin() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("0.5".into(), false);

    let _ = engine.add_item_to_stack("1".into(), false);

    // evaluate from last stack entries to first

    // 1 = pi/2
    let _ = engine.asin();
    assert_eq!(
        format!("{:.7}", engine.get_operands_as_f(1).unwrap()[0]),
        format!("{:.7}", PI / 2.0)
    );

    // 0.5 = pi/6
    let _ = engine.asin();
    assert_eq!(
        format!("{:.7}", engine.get_operands_as_f(1).unwrap()[0]),
        format!("{:.7}", PI / 6.0)
    );
}

#[test]
fn test_acos() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("0.5".into(), false);

    let _ = engine.add_item_to_stack("1".into(), false);

    // evaluate from last stack entries to first

    // 1 = 0
    let _ = engine.acos();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);

    // 0.5 = pi/3
    let _ = engine.acos();
    assert_eq!(
        format!("{:.7}", engine.get_operands_as_f(1).unwrap()[0]),
        format!("{:.7}", PI / 3.0)
    );
}

#[test]
fn test_atan() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("0".into(), false);

    let _ = engine.add_item_to_stack("1".into(), false);

    // evaluate from last stack entries to first

    // 1 = pi/4
    let _ = engine.atan();
    assert_eq!(
        format!("{:.7}", engine.get_operands_as_f(1).unwrap()[0]),
        format!("{:.7}", PI / 4.0)
    );

    // 0 = 0
    let _ = engine.atan();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);
}

#[test]
fn test_chs() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-9".into(), false);
    let _ = engine.add_item_to_stack("4".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.chs();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], -4.0);

    let _ = engine.chs();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 9.0);
}

#[test]
fn test_log() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("10".into(), false);
    let _ = engine.add_item_to_stack("100".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.log();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 2.0);

    let _ = engine.log();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0);
}

#[test]
fn test_logb() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("27".into(), false);
    let _ = engine.add_item_to_stack("3".into(), false);

    let _ = engine.add_item_to_stack("8".into(), false);
    let _ = engine.add_item_to_stack("2".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.logb();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 3.0);

    let _ = engine.logb();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 3.0);
}

#[test]
fn test_ln() {
    let mut engine = Engine::new();

    // TODO: fix variable storing as decimals
    let _ = engine.add_item_to_stack("#e".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.ln();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0);
}

#[test]
fn test_abs() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-1".into(), false);
    let _ = engine.add_item_to_stack("0".into(), false);
    let _ = engine.add_item_to_stack("1".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.abs();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0);

    let _ = engine.abs();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);

    let _ = engine.abs();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0);
}

#[test]
fn test_eq() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-1".into(), false);
    let _ = engine.add_item_to_stack("-1".into(), false);

    let _ = engine.add_item_to_stack("1".into(), false);
    let _ = engine.add_item_to_stack("2".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.eq();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);

    let _ = engine.eq();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0);
}

#[test]
fn test_gt() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-1".into(), false);
    let _ = engine.add_item_to_stack("0".into(), false);

    let _ = engine.add_item_to_stack("2".into(), false);
    let _ = engine.add_item_to_stack("1".into(), false);

    let _ = engine.add_item_to_stack("2".into(), false);
    let _ = engine.add_item_to_stack("2".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.gt();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);

    let _ = engine.gt();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0);

    let _ = engine.gt();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);
}

#[test]
fn test_lt() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-1".into(), false);
    let _ = engine.add_item_to_stack("0".into(), false);

    let _ = engine.add_item_to_stack("2".into(), false);
    let _ = engine.add_item_to_stack("1".into(), false);

    let _ = engine.add_item_to_stack("2".into(), false);
    let _ = engine.add_item_to_stack("2".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.lt();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);

    let _ = engine.lt();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);

    let _ = engine.lt();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0);
}

#[test]
fn test_gte() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-1".into(), false);
    let _ = engine.add_item_to_stack("0".into(), false);

    let _ = engine.add_item_to_stack("2".into(), false);
    let _ = engine.add_item_to_stack("1".into(), false);

    let _ = engine.add_item_to_stack("2".into(), false);
    let _ = engine.add_item_to_stack("2".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.gte();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0);

    let _ = engine.gte();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0);

    let _ = engine.gte();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);
}

#[test]
fn test_lte() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-1".into(), false);
    let _ = engine.add_item_to_stack("0".into(), false);

    let _ = engine.add_item_to_stack("2".into(), false);
    let _ = engine.add_item_to_stack("1".into(), false);

    let _ = engine.add_item_to_stack("2".into(), false);
    let _ = engine.add_item_to_stack("2".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.lte();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0);

    let _ = engine.lte();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);

    let _ = engine.lte();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0);
}

#[test]
fn test_round() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-1.2".into(), false);
    let _ = engine.add_item_to_stack("0.1".into(), false);
    let _ = engine.add_item_to_stack("2.6".into(), false);
    let _ = engine.add_item_to_stack("-1.6".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.round();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], -2.0);

    let _ = engine.round();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 3.0);

    let _ = engine.round();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.0);

    let _ = engine.round();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], -1.0);
}

#[test]
fn test_invert() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-3".into(), false);
    let _ = engine.add_item_to_stack("3".into(), false);
    let _ = engine.add_item_to_stack("-4".into(), false);
    let _ = engine.add_item_to_stack("4".into(), false);

    // evaluate from last stack entries to first
    let _ = engine.invert();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 0.25);

    let _ = engine.invert();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], -0.25);

    let _ = engine.invert();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], 1.0 / 3.0);

    let _ = engine.invert();
    assert_eq!(engine.get_operands_as_f(1).unwrap()[0], -1.0 / 3.0);
}

#[test]
fn test_drop() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-1.2".into(), false);
    let _ = engine.add_item_to_stack("0.1".into(), false);

    let _ = engine.drop();
    assert_eq!(engine.stack, vec![Bucket::from(-1.2),]);

    let _ = engine.drop();
    assert_eq!(engine.stack, vec![]);

    let result = engine.drop();
    assert_eq!(
        result,
        Ok(squiid_engine::protocol::MessageAction::SendStack)
    );
}

#[test]
fn test_swap() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-1.2".into(), false);
    let _ = engine.add_item_to_stack("0.1".into(), false);
    let _ = engine.add_item_to_stack("3".into(), false);

    let _ = engine.swap();
    assert_eq!(
        engine.stack,
        vec![Bucket::from(-1.2), Bucket::from(3), Bucket::from(0.1),]
    );
}

#[test]
fn test_dup() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-1.2".into(), false);
    let _ = engine.add_item_to_stack("0.1".into(), false);

    let _ = engine.dup();
    assert_eq!(
        engine.stack,
        vec![Bucket::from(-1.2), Bucket::from(0.1), Bucket::from(0.1),]
    );
}

#[test]
fn test_roll_down() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-1.2".into(), false);
    let _ = engine.add_item_to_stack("0.1".into(), false);
    let _ = engine.add_item_to_stack("3".into(), false);

    let _ = engine.roll_down();
    assert_eq!(
        engine.stack,
        vec![Bucket::from(3), Bucket::from(-1.2), Bucket::from(0.1),]
    );

    let _ = engine.clear();
    let result = engine.roll_down();

    assert!(matches!(result, Err(_)));
}

#[test]
fn test_roll_up() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-1.2".into(), false);
    let _ = engine.add_item_to_stack("0.1".into(), false);
    let _ = engine.add_item_to_stack("3".into(), false);

    let _ = engine.roll_up();
    assert_eq!(
        engine.stack,
        vec![Bucket::from(0.1), Bucket::from(3), Bucket::from(-1.2),]
    );

    let _ = engine.clear();
    let result = engine.roll_up();

    assert!(matches!(result, Err(_)));
}

#[test]
fn test_store() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("-1.2".into(), false);
    let _ = engine.add_item_to_stack("0".into(), false);

    let _ = engine.add_item_to_stack("-1.2".into(), false);
    let _ = engine.add_item_to_stack("a".into(), false);

    // test valid variable assignment
    let _ = engine.store();
    assert_eq!(*engine.variables.get("a").unwrap(), Bucket::from(-1.2));

    // test invalid variable assignment
    let result = engine.store();
    assert!(matches!(result, Err(_)));
}

#[test]
fn test_purge() {
    let mut engine = Engine::new();

    engine
        .variables
        .insert(String::from("a"), Bucket::from(-1.2));

    let _ = engine.add_item_to_stack("a".into(), false);
    let _ = engine.add_item_to_stack("a".into(), false);

    // test for variable presence
    assert_eq!(*engine.variables.get("a").unwrap(), Bucket::from(-1.2));

    let _ = engine.purge();

    // test that variable was deleted
    assert!(engine.variables.get("a").is_none());

    // test invalid variable deletion
    let result = engine.purge();
    assert!(matches!(result, Err(_)));
}

#[test]
fn test_invstore() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("a".into(), false);
    let _ = engine.add_item_to_stack("-1.2".into(), false);

    let _ = engine.invstore();

    // test for variable presence
    assert_eq!(*engine.variables.get("a").unwrap(), Bucket::from(-1.2));
}

#[test]
fn test_clear() {
    let mut engine = Engine::new();

    let _ = engine.add_item_to_stack("23".into(), false);
    let _ = engine.add_item_to_stack("-1.2".into(), false);
    let _ = engine.add_item_to_stack("test".into(), false);

    assert!(!engine.stack.is_empty());
    let _ = engine.clear();
    assert!(engine.stack.is_empty());
}

#[test]
fn test_undo() {
    macro_rules! push_to_history {
        ($engine:ident) => {
            $engine.history.push_back($engine.stack.clone());
            $engine
                .variable_history
                .push_back($engine.variables.clone());
        };
    }

    let mut engine = Engine::new();

    // after each command, we must push a copy of the stack to the engine history

    let _ = engine.add_item_to_stack("1".into(), false);
    push_to_history!(engine);
    let _ = engine.add_item_to_stack("2".into(), false);
    push_to_history!(engine);
    let _ = engine.add_item_to_stack("test".into(), false);
    push_to_history!(engine);

    // test undo of adding something to the stack
    let _ = engine.undo();
    push_to_history!(engine);
    assert_eq!(engine.stack, vec![Bucket::from(1), Bucket::from(2),]);

    // test undo of operation
    let _ = engine.add();
    push_to_history!(engine);
    assert_eq!(engine.stack, vec![Bucket::from(3),]);

    let _ = engine.undo();
    push_to_history!(engine);
    assert_eq!(engine.stack, vec![Bucket::from(1), Bucket::from(2),]);

    // test undo of clear
    let _ = engine.clear();
    push_to_history!(engine);
    assert_eq!(engine.stack, vec![]);

    let _ = engine.undo();
    push_to_history!(engine);
    assert_eq!(engine.stack, vec![Bucket::from(1), Bucket::from(2),]);

    // test undo of variable assignment
    let _ = engine.add_item_to_stack("a".into(), false);
    push_to_history!(engine);
    let _ = engine.store();
    push_to_history!(engine);
    assert_eq!(engine.stack, vec![Bucket::from(1),]);
    assert_eq!(*engine.variables.get("a").unwrap(), Bucket::from(2));

    let _ = engine.undo();
    push_to_history!(engine);
    assert_eq!(
        engine.stack,
        vec![Bucket::from(1), Bucket::from(2), Bucket::from("a"),]
    );
    assert_eq!(engine.variables.get("a"), None);
}

#[test]
fn test_list_commands() {
    let mut engine = Engine::new();

    assert!(matches!(
        engine.list_commands().unwrap(),
        squiid_engine::protocol::MessageAction::SendCommands
    ));
}

#[test]
fn test_prev_ans() {
    let mut engine = Engine::new();

    assert_eq!(engine.previous_answer, Bucket::from(0));

    // simulate adding a function answer
    let _ = engine.add_item_to_stack("10".into(), true);
    assert_eq!(engine.previous_answer, Bucket::from(10));

    // test that an operation uses it correctly
    let _ = engine.add_item_to_stack("@".into(), false);
    let _ = engine.add_item_to_stack("5".into(), false);
    let _ = engine.subtract();

    assert_eq!(*engine.stack.last().unwrap(), Bucket::from(5));
    assert_eq!(engine.previous_answer, Bucket::from(5));
}
