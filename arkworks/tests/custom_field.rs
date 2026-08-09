use arkworks::custom_field::CustomField;

#[test]
fn test_custom_field() {
    let a: CustomField = CustomField::from(2u64);
    let b: CustomField = CustomField::from(3u64);
    let c = a + b;

    assert_eq!(c, CustomField::from(5u64));
}
