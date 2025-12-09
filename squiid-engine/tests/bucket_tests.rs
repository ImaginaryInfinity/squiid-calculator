use squiid_engine::bucket::{Bucket, BucketTypes, CONSTANT_IDENTIFIERS, ConstantTypes};

#[test]
fn test_bucket_creation() {
    assert_eq!(Bucket::from(3.0).bucket_type, BucketTypes::Float);

    assert_eq!(Bucket::from(3).bucket_type, BucketTypes::Float);

    assert_eq!(Bucket::from("3").bucket_type, BucketTypes::String);

    assert_eq!(
        Bucket::from(String::from("3")).bucket_type,
        BucketTypes::String
    );

    assert_eq!(Bucket::from(3_u8).bucket_type, BucketTypes::Float);

    assert_eq!(Bucket::from(3_i64).bucket_type, BucketTypes::Float);

    assert_eq!(Bucket::from(-3).bucket_type, BucketTypes::Float);

    assert_eq!(Bucket::from(-3.0).bucket_type, BucketTypes::Float);

    assert_eq!(Bucket::new_undefined().bucket_type, BucketTypes::Undefined);
}

#[test]
fn test_convert_to_string() {
    assert_eq!(Bucket::from("test").to_string(), String::from("test"));
    assert_eq!(Bucket::from(3).to_string(), String::from("3"));
}

#[test]
fn test_constants_have_string_repr() {
    let all_variants = [
        ConstantTypes::Pi,
        ConstantTypes::HalfPi,
        ConstantTypes::ThirdPi,
        ConstantTypes::QuarterPi,
        ConstantTypes::SixthPi,
        ConstantTypes::EighthPi,
        ConstantTypes::TwoPi,
        ConstantTypes::E,
        ConstantTypes::C,
        ConstantTypes::G,
        ConstantTypes::Phi,
    ];

    let defined_variants: Vec<ConstantTypes> = CONSTANT_IDENTIFIERS.values().copied().collect();

    for variant in all_variants {
        // check that each variant has a string representation in the hashmap
        assert!(
            defined_variants.contains(&variant),
            "Missing ConstantType in identifier map: {:?}",
            variant
        );
    }
}
