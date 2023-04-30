use squiid_engine::bucket::{Bucket, BucketTypes};

#[test]
fn test_bucket_creation() {
    assert_eq!(Bucket::from(3.0).bucket_type, BucketTypes::Float);

    assert_eq!(Bucket::from(3).bucket_type, BucketTypes::Float);

    assert_eq!(Bucket::from("3").bucket_type, BucketTypes::String);

    assert_eq!(
        Bucket::from(String::from("3")).bucket_type,
        BucketTypes::String
    );

    assert_eq!(Bucket::from(3 as u8).bucket_type, BucketTypes::Float);

    assert_eq!(Bucket::from(3 as i64).bucket_type, BucketTypes::Float);

    assert_eq!(Bucket::from(-3).bucket_type, BucketTypes::Float);

    assert_eq!(Bucket::from(-3.0).bucket_type, BucketTypes::Float);

    assert_eq!(Bucket::new_undefined().bucket_type, BucketTypes::Undefined);
}

#[test]
fn test_convert_to_string() {
    assert_eq!(Bucket::from("test").to_string(), String::from("test"));
    assert_eq!(Bucket::from(3).to_string(), String::from("3"));
}
