use super::super::codec::utils;
use bencodex::json::{BinaryEncoding, from_json_string};

#[test]
fn spec_test_base64() {
    let specs = utils::iter_spec_with_json(BinaryEncoding::Base64).unwrap();
    for spec in specs {
        println!("---- SPEC [{}] ----", spec.name);

        println!("JSON: {:?}", spec.json);
        assert_eq!(from_json_string(&spec.json).unwrap(), spec.bvalue);

        println!("---- PASSED ----");
    }
}

#[test]
fn spec_test_hex() {
    let specs = utils::iter_spec_with_json(BinaryEncoding::Hex).unwrap();
    for spec in specs {
        println!("---- SPEC [{}] ----", spec.name);

        println!("JSON: {:?}", spec.json);
        assert_eq!(from_json_string(&spec.json).unwrap(), spec.bvalue);

        println!("---- PASSED ----");
    }
}
