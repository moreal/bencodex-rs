use super::super::codec::utils;
use bencodex::json::{BinaryEncoding, JsonEncodeOptions, to_json_with_options};

const SPEC_TEST_BASE64_OPTIONS: JsonEncodeOptions = JsonEncodeOptions {
    binary_encoding: BinaryEncoding::Base64,
};

const SPEC_TEST_HEX_OPTIONS: JsonEncodeOptions = JsonEncodeOptions {
    binary_encoding: BinaryEncoding::Hex,
};

#[test]
fn spec_test_base64() {
    let specs = utils::iter_spec_with_json(BinaryEncoding::Base64).unwrap();
    for spec in specs {
        println!("---- SPEC [{}] ----", spec.name);

        println!("JSON: {:?}", spec.json);
        assert_eq!(
            to_json_with_options(&spec.bvalue, SPEC_TEST_BASE64_OPTIONS)
                .expect("Failed to encode JSON."),
            spec.json
        );

        println!("---- PASSED ----");
    }
}

#[test]
fn spec_test_hex() {
    let specs = utils::iter_spec_with_json(BinaryEncoding::Hex).unwrap();
    for spec in specs {
        println!("---- SPEC [{}] ----", spec.name);

        println!("JSON: {:?}", spec.json);
        assert_eq!(
            to_json_with_options(&spec.bvalue, SPEC_TEST_HEX_OPTIONS)
                .expect("Failed to encode JSON."),
            spec.json
        );

        println!("---- PASSED ----");
    }
}
